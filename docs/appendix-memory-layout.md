# Appendix: Memory Layout - Where Your Data Lives

This document demystifies where your Rust data actually lives in memory. We'll visualize the process memory layout and understand the stack, heap, and static data segments.

**Recommended resource:** [cheats.rs/#memory-layout](https://cheats.rs/#memory-layout) provides excellent visual memory layouts for Rust types.

## The Simple Program

Let's start with a concrete Rust program and trace where everything lives:

```rust
// Global/static data - lives in data segment
static GREETING: &str = "Hello";
static mut BUFFER: [u8; 10_000] = [0; 10_000];  // 10 KB zero-initialized

fn main() {
    // Stack: local variables
    let x = 42;
    let y = 100;

    // Stack: String struct (24 bytes: ptr + len + cap)
    // Heap: actual string data "world"
    let s = String::from("world");

    // Stack: vector struct (24 bytes: ptr + len + cap)
    // Heap: array data [1, 2, 3, 4, 5]
    let v = vec![1, 2, 3, 4, 5];

    // Stack: native array - all data lives on stack (20 bytes)
    let arr = [10, 20, 30, 40, 50];

    // Stack: function call frame
    // Passing by value (x) and by reference (&s)
    let doubled = process_data(x, &s);

    println!("{} -> {}", x, doubled);
}

fn process_data(param_num: i32, param_text: &String) -> i32 {
    // Stack: new function frame
    // Arguments passed:
    // - param_num: COPY of x's value (42) - passed by value
    // - param_text: pointer to s (on stack) - passed by reference
    let result = param_num * 2;
    println!("{}: {}", param_text, result);

    // Return: result is COPIED to caller's stack frame
    result  // Returns 84
}
```

Now let's see where each piece of data lives in memory.

## Process Memory Layout

When your Rust program runs, the operating system gives it a contiguous chunk of virtual memory organized into distinct regions:

```
High Memory Addresses (0x0000_7FFF_FFFF_FFFF - User Space upper bound)
┌─────────────────────────────────────────────┐
│                                             │
│              STACK                          │  ← Grows downward
│  (Function frames, local variables)         │
│                                             │
├─────────────────────────────────────────────┤
│                   ↓                         │
│                                             │
│                                             │
│              (unused space)                 │
│                                             │
│                                             │
│                   ↑                         │
├─────────────────────────────────────────────┤
│                                             │
│              HEAP                           │  ← Grows upward
│  (Dynamically allocated: Box, Vec, String)  │
│                                             │
├─────────────────────────────────────────────┤
│                                             │
│        BSS (Uninitialized Data)             │
│  (static mut with no initializer)           │
│                                             │
├─────────────────────────────────────────────┤
│                                             │
│        DATA (Initialized Data)              │
│  (static, const, string literals)           │
│                                             │
├─────────────────────────────────────────────┤
│                                             │
│              TEXT (Code)                    │
│  (Your compiled functions)                  │
│                                             │
└─────────────────────────────────────────────┘
Low Memory Addresses (0x0000_0000_0000_0000)
```

**Key Insight:** The stack and heap grow toward each other!

## Let's Trace Our Program

Now let's see exactly where each piece of data from our example lives.

### Step 1: Program Starts - Static Data is Loaded

Before `main()` even runs, the OS loads static data into the DATA segment:

```
TEXT Segment (Code):
  0x1000: fn main() { ... }
  0x2000: fn process_data() { ... }
  0x3000: println!() code
  ...

DATA Segment (Initialized Statics):
  0x5000: GREETING = "Hello" (string literal)
          ├─ ptr:  0x5000  ─┐
          ├─ len:  5        │
          └─ "Hello\0"  <───┘

BSS Segment (Zero-Initialized Statics):
  0x6000: BUFFER = [0u8; 10_000]  (10 KB of zeros)
          [0][0][0][0]...[0][0][0][0]
          (10,000 bytes - all initialized to 0 at program start)

HEAP: (empty at start)

STACK: (empty at start)
```

> **Why BSS exists:** It's a file size optimization! BSS stores only zeros, so the executable doesn't need to include them.
>
> **Program in file:** `static BUFFER: [u8; 1_000_000] = [0; 1_000_000];`
>
> - BSS: Executable just says "allocate 1 MB of zeros" (~16 bytes metadata)
> - Data: Would need to store all 1 million bytes of zeros (~1 MB in file)
>
> **Process in memory:** Both are 1 MB of zeros in memory. The OS allocates and zeroes the BSS memory at load time.
>
> **Result:** Executable with BSS is ~320 KB, with Data would be ~1.3 MB. Same memory usage at runtime, but different file sizes!

### Step 2: main() Starts - Stack Frame Created

When `main()` is called, a **stack frame** is created:

```
STACK (grows downward from high addresses):
┌────────────────────────────────────┐  ← Stack Pointer (SP)
│                                    │    0x7FFF_FFFF_FFF0
│    main()'s stack frame            │
│                                    │
│  [Return address to OS]            │  ← Where to return after main
│  [Saved registers]                 │
│                                    │
│  x: i32 = 42                       │  ← Local variable (4 bytes)
│  y: i32 = 100                      │  ← Local variable (4 bytes)
│                                    │
│  s: String                         │  ← String struct (24 bytes):
│    ├─ ptr:  0x8000 ──────┐         │     Points to heap
│    ├─ len:  5            │         │
│    └─ cap:  5            │         │
│                          │         │
│  v: Vec<i32>             │         │  ← Vec struct (24 bytes):
│    ├─ ptr:  0x8100 ───┐  │         │     Points to heap
│    ├─ len:  5         │  │         │
│    └─ cap:  5         │  │         │
│                       │  │         │
│  arr: [i32; 5]        │  │         │  ← Native array (20 bytes):
│    [50]               │  │         │     All data on stack!
│    [40]               │  │         │     Elements at increasing
│    [30]               │  │         │     addresses (50 highest,
│    [20]               │  │         │     10 lowest)
│    [10]               │  │         │
│                       │  │         │
└───────────────────────┼──┼─────────┘
                        │  │
                        │  └───────┐
HEAP (grows upward):    │          │
┌───────────────────────┼──────────┼─────────────┐
│                       ↓          │             │
│  0x8100: [1][2][3][4][5]         │             │
│   v's data (6 bytes)             │             │
│                                  ↓             │
│                         0x8000: [w][o][r][l][d]│  ← v's data
│               (20 bytes: 5 * 4-byte integers)  │
│                                                │
└────────────────────────────────────────────────┘
```

**Important observations:**

1. **`x` and `y`** are just 4 bytes each, living directly on the stack
2. **`s` (String)** is 24 bytes on the stack (metadata: pointer, length, capacity)
   - The **actual string data** "world" lives on the heap
3. **`v` (Vec)** is 24 bytes on the stack (metadata: pointer, length, capacity)
   - The **actual array data** [1,2,3,4,5] lives on the heap
4. **`arr` (native array)** is 20 bytes entirely on the stack (no heap allocation!)
   - All 5 integers live directly in the array, no pointer indirection

### Step 3: Calling process_data() - New Stack Frame and Passing Arguments

When we call `process_data(x, &s)`, here's what the CPU actually does (x86-64 calling convention):

1. **Arguments loaded into registers** (not pushed to stack!):
   - `x` (i32, 4 bytes): Loaded into `EDI` register → becomes `param_num`
   - `&s` (&String, 8 bytes): Pointer loaded into `RSI` register → becomes `param_text`
   - **There is no param_num or param_text in memory** - they ARE the registers themselves
2. **CALL instruction executes**: Pushes return address onto stack, then jumps to process_data
3. **Callee (process_data) sets up its stack frame**:
   - Saves registers if needed
   - Allocates space for local variables
   - May spill register arguments to stack (compiler's choice)

```
CPU REGISTERS (not in memory!):
┌────────────────────────────────────┐
│  RBP:  0x7FFF_FFFF_FF00            │  Base pointer (main's frame base)
│  RSP:  0x7FFF_FFFF_FE00            │  Stack pointer (current top)
│  EDI:  42          ← param_num     │  Arguments passed via registers!
│  RSI:  0x7FFF...   ← param_text    │  Points to s on stack
└──────────┼─────────────────────────┘  These are NOT in stack memory
           │
STACK:     └ ─ ─ ─ ─ ─ ─ ─ ┐
┌──────────────────────────┼─────────┐  ← main() pushed first (higher address)
│                                    │    0x7FFF_FFFF_FFF0
│    main()'s stack frame  │         │
│                                    │
│  [Return address to OS]  │         │
│  [Saved main's RBP]                │  ← RBP points here (0x7FFF_FFFF_FF00)
│                          │         │
│  x: i32 = 42                       │  ← at [rbp-4]
│  y: i32 = 100            │         │  ← at [rbp-8]
│  [padding ~24 bytes]               │  ← Compiler adds padding for alignment
│  s: String  ←─ ─ ─ ─ ─ ─ ┘         │  ← at [rbp-32] (aligned to 8-byte boundary)
│    ├─ ptr:  0x8000  ───────┐       │
│    ├─ len:  5              │       │
│    └─ cap:  5              │       │
│                            │       │
│  v: Vec<i32>               │       │
│    ├─ ptr:  0x8100  ──┐    │       │
│    ├─ len:  5         │    │       │
│    └─ cap:  5         │    │       │
│                       │    │       │
│  arr: [i32; 5]        │    │       │
│    [50]               │    │       │
│    [40]               │    │       │
│    [30]               │    │       │
│    [20]               │    │       │
│    [10]               │    │       │
│                       │    │       │
│  doubled: i32 = ???   │    │       │  ← Space for return value (not set yet)
│                       │    │       │
├───────────────────────┼────┼───────┤
│  [Return address to main]  │       │  ← process_data()'s stack frame
│  [Saved RBP = 0x7FFF_FFFF_FF00]    │  ← "push rbp" saved it here
│                       │    │       │  ← process_data's RBP points HERE
│  result: i32 = 84     │    │       │  ← Local variable at [rbp-4]
│                       │    │       │
│  (param_num/param_text NOT here!)  │  ← Arguments are in REGISTERS, not stack!
│                       │    │       │
│  [allocated space]    │    │       │  ← "sub rsp, 16" allocated this
│                       │    │       │
└───────────────────────│────┼───────┘  ← RSP points here (0x7FFF_FFFF_FE00)
                        │    │
HEAP:                   │    │
┌───────────────────────┼────┼─────────────────┐
│                       │    ↓                 │
│  0x8000: "world\0"    │  [w][o][r][l][d][\0] │
│                       ↓                      │
│              0x8100: [1][2][3][4][5]         │
│                                              │
└──────────────────────────────────────────────┘
```

**Key observations about arguments and returns:**

1. **`param_num` and `param_text` are in CPU REGISTERS, not in memory!**
   - `param_num` (value 42) lives in the EDI register
   - `param_text` (pointer to s) lives in the RSI register
   - They are NOT stored on the stack (unless the compiler decides to spill them later)

2. **Pass by value vs by reference both use registers**:
   - Pass by value (`x`): The value 42 is copied into EDI register
   - Pass by reference (`&s`): The pointer to s (address on stack) is copied into RSI register

3. **`doubled` variable** in main's frame has space allocated but isn't set yet - it will receive the return value

4. **Stack frames stack up** - process_data's frame sits on top of main's frame

5. **The heap data doesn't move** - only stack frames are created/destroyed

6. **`arr` stays on the stack** in main's frame - native arrays don't involve heap

**How arguments are actually passed (x86-64 System V ABI):**

```asm
; Conceptual assembly for: let doubled = process_data(x, &s);

mov    edi, DWORD PTR [rbp-4]    ; Load x (42), located at rbp-4 into EDI register
lea    rsi, [rbp-32]             ; Load address of s into RSI register (pointer to s on stack)
call   process_data              ; CALL pushes return address, jumps to function
```

Inside `process_data`

```asm

; Arguments are in registers: EDI = 42, RSI = pointer to s

; Function prologue - setting up stack frame:
push   rbp                       ; Save caller's base pointer
mov    rbp, rsp                  ; Set up our base pointer
sub    rsp, 16                   ; Allocate space for local variables (result, etc.)

; Now we can execute the function body:
; let result = param_num * 2;
mov    eax, edi                  ; Load param_num (42) into EAX
shl    eax, 1                    ; Multiply by 2 (shift left) -> EAX = 84
mov    DWORD PTR [rbp-4], eax    ; Store result on stack

; ... println! call happens here ...

; Return preparation:
mov    eax, DWORD PTR [rbp-4]    ; Load result (84) into EAX (return register)

; Function epilogue - cleanup:
add    rsp, 16                   ; Deallocate local variables
pop    rbp                       ; Restore caller's base pointer
ret                              ; Return to caller (pops return address, jumps back)
```

**Key points:**

1. **Arguments go into registers first** (not pushed to stack):
   - First 6 integer/pointer arguments use: RDI, RSI, RDX, RCX, R8, R9
   - Our i32 uses EDI (lower 32 bits of RDI)
   - Our &String uses RSI (just a single pointer, 8 bytes)

2. **CALL instruction** pushes return address onto stack automatically

3. **Compiler may spill to stack** if:
   - Register needed for other operations
   - Function has too many arguments (7+ integers)
   - Debugging is enabled (makes variables inspectable)

4. **Pass by value vs by reference** both use registers - the difference is what's copied:
   - **By value (`x`)**: The actual value (42) is copied into EDI
   - **By reference (`&s`)**: Only the pointer to s (8 bytes) is copied into RSI, pointing to s on the stack

### Step 4: process_data() Returns - Value Copied Back and Stack Frame Destroyed

When `process_data()` returns, two things happen:

1. **Return value is copied**: The value in `result` (84) is **copied** to `doubled` in main's frame (using CPU register or direct memory copy)
2. **Stack frame is popped**: process_data's entire frame is destroyed

```
STACK:
┌───────────────────────────────────┐
│                                   │
│    main()'s stack frame           │
│                                   │
│  [Return address to OS]           │
│  [Saved registers]                │
│                                   │
│  x: i32 = 42                      │
│  y: i32 = 100                     │
│                                   │
│  s: String                        │
│    ├─ ptr:  0x8000  ──────┐       │
│    ├─ len:  5             │       │
│    └─ cap:  5             │       │
│                           │       │
│  v: Vec<i32>              │       │
│    ├─ ptr:  0x8100  ──┐   │       │
│    ├─ len:  5         │   │       │
│    └─ cap:  5         │   │       │
│                       │   │       │
│  arr: [i32; 5]        │   │       │
│    [50]               │   │       │
│    [40]               │   │       │
│    [30]               │   │       │
│    [20]               │   │       │
│    [10]               │   │       │
│                       │   │       │
│  doubled: i32 = 84    │   │       │  ← Return value COPIED here (4 bytes)
│                       │   │       │
└───────────────────────┼───┼───────┘
                        │   │
                        │   └────────────┐
HEAP:                   │                │
┌───────────────────────┼────────────────┼───┐
│                       ↓                ↓   │
│  0x8000: "world\0"    [w][o][r][l][d][\0]  │
│                                            │
│  0x8100: [1][2][3][4][5]                   │
│                                            │
└────────────────────────────────────────────┘
```

**Key observations about returns:**

1. **Return value is copied**: The 4 bytes of `result` are copied (typically via CPU register like `RAX` on x86-64, then to stack)
2. **process_data's stack frame is gone**: All local variables (param_num, param_text, result) are destroyed
3. **The heap data remains untouched**: Only stack frames change, heap is unaffected
4. **doubled now has the value 84**: Ready to be used by main

**How return values work:**

- **Small values** (like i32, 4 bytes): Returned via CPU register (RAX on x86-64), then copied to destination
- **Larger values** (like structs): Caller pre-allocates space, callee writes directly to it
- **Owned heap types** (like Vec, String): Only the metadata is copied (24 bytes), heap data stays put

### Step 5: main() Ends - Cleanup

When `main()` returns, `s` and `v` go out of scope. Their `Drop` implementations run:

1. **`s` is dropped**: Calls `dealloc()` to free the heap memory at 0x8000
2. **`v` is dropped**: Calls `dealloc()` to free the heap memory at 0x8100
3. **main's stack frame is popped**: All local variables disappear

```
STACK: (empty)

HEAP: (freed)
  0x8000: (deallocated)
  0x8100: (deallocated)

DATA Segment: (still there)
  0x5000: GREETING = "Hello"

BSS Segment: (still there)
  0x6000: COUNTER = 0
```

## Memory Regions in Detail

### The Stack

**What lives here:**

Before we categorize by type, let's ask: **Which of these types live on the stack?**

- Primitives: `i32`, `f64`, `bool`, `char`?
- Enums?
- Structs?
- Arrays: `[T; N]`?
- Pointers: `&T`, `&mut T`, `*const T`, `*mut T`?
- Smart pointers: `String`, `Vec`, `Box` (which are actually just structs)?

You might think: "Primitives live on stack, arrays live on heap..."

**But actually, the type doesn't matter.** Here's the simple rule:

**The Rule: All local variables (declared with `let`) live on the stack.**

Let's test this. Which of these live on the stack?

```rust
let x: i32 = 0;

struct Number {
    n: i32
}

let arr: [i32; 3] = [1, 2, 3];
```

**Answer: The values declared with `let` live on stack.**

- `x` lives on stack (it's a local variable)
- `struct Number { n: i32 }` is just a type definition - doesn't live anywhere!
- `arr` lives on stack (it's a local variable)

When we create an _instance_ of `Number` with `let`, that's when it gets memory:

```rust
let num = Number { n: 42 };  // num lives on stack, so field n (as part of num) lives on stack
```

**No matter the type, if it's a local variable, it lives on the stack:**

```rust
let result = Ok(42);           // enum on stack (including its data)
let n: i32 = 5;                // primitive on stack
let ref_n: &i32 = &n;          // reference (pointer) on stack, points to n (also on stack)
let p_n: *const i32 = &n;      // raw pointer on stack, points to n (also on stack)
```

> **Note:** Pointers are bridges between stack and heap. They can point to stack (like `ref_n` above) or to heap (like Vec's internal `ptr`). We'll explore heap allocation in detail later.

**What about `String`, `Vec`, `Box`?**

These are also just structs! Let's see what `Vec` actually is:

```rust
struct Vec<T> {
    ptr: *mut T,  // pointer to heap data
    len: usize,   // length
    cap: usize,   // capacity
}

let number = Number { n: 42 };  // number on stack, field n on stack
let v = Vec::new();             // v (the struct) on stack
                                // ptr is null/dangling, len=0, cap=0
v.push(1);                      // FIRST push calls alloc()!
                                // Now ptr points to heap, len=1, cap=4 (typically)
v.push(2);                      // adds to heap, len=2, cap=4
```

**Key insight:** The `Vec` struct itself always lives on stack. Its fields (`ptr`, `len`, `cap`) always live on stack. But `ptr` only points to heap **after the first allocation** (which happens during the first `push()` that needs capacity).

- After `Vec::new()`: ptr is null (or dangling), no heap allocation yet
- After first `push(1)`: ptr points to heap (alloc() was called), heap data exists

**Stack memory layout:**

```
Stack:
┌────────────────────────────┐
│ number: Number             │
│   n: 42                    │  4 bytes
├────────────────────────────┤
│ v: Vec<i32>                │
│   ptr:  0x1000  ─────┐     │  8 bytes (pointer)
│   len:  2            │     │  8 bytes
│   cap:  4            │     │  8 bytes
└──────────────────────┼─────┘  Total: 24 bytes on stack
                       │
                       └──────> Heap at 0x1000: [1][2]  (8 bytes + capacity for 2 more)
```

We'll explore heap and allocation in more detail in the next section.

**Summary:**

**Your local variables (you create these):**

- **The Rule**: Everything declared with `let` in a function lives on stack - type doesn't matter
- Primitives: `i32`, `f64`, `bool`, `char`
- Structs: entire struct including all fields
- Enums: including their variant data
- Arrays: `[T; N]` - all elements inline
- Pointers: `&T`, `&mut T`, `*const T`, `*mut T` - the pointer itself (8 bytes)
- Smart pointer structs: `String`, `Vec`, `Box` - the struct metadata on stack, the data they point to on heap

**Compiler-managed (you don't interact with these):**

- Function parameters (passed via registers, may be spilled to stack)
- Return addresses (managed via `CALL`/`RET` instructions)
- Saved registers (managed via `PUSH`/`POP` instructions)

**Characteristics:**

- **Automatic management**: Variables automatically disappear when they go out of scope. The CPU has built-in stack instructions (`PUSH`, `POP`, `CALL`, `RET`) and a dedicated stack pointer register (`RSP` on x86-64) that make stack operations trivial.
- **Fast allocation**: Just move the stack pointer (one CPU instruction: `sub rsp, 16` to allocate 16 bytes)
- **Fixed size**: Typically 2-8 MB (OS-dependent)
- **LIFO (Last In, First Out)**: Like a stack of plates
- **Grows downward**: From high addresses to low addresses

**Example:**

```rust
fn example() {
    let x = 42;        // Allocate 4 bytes on stack
    let y = vec![1];   // Allocate 24 bytes on stack (Vec metadata)
}  // Stack pointer moves back, x and y are gone
```

**Stack overflow** happens when you use too much stack space:

```rust
fn infinite_recursion() {
    let huge = [0u8; 1_000_000];  // 1 MB per call!
    infinite_recursion();         // Each call adds another frame
}
// Eventually: stack overflow!
```

### Raw Pointers

**References: De-abstracting the abstraction**

First, let's understand what references actually are. Despite all the Rust jargon about "borrowing" and "lifetimes", references are just **pointers** - plain old memory addresses.

```rust
let mut x: i32 = 42;          // x lives on stack (4 bytes)
let x_ref: &i32 = &x;         // x_ref is a pointer to x (8 bytes on 64-bit)
println!("x is at address: {:p}", x_ref);  // Prints something like: 0x00007ffc1234abcd

// Can't have both immutable and mutable references at the same time:
let x_mut_ref: &mut i32 = &mut x;  // ❌ Error: x_ref is still in scope
```

You can think a reference as a safe pointer guaranteed by the compiler.

**What's in memory :**

```
Stack (User Space - lower canonical addresses start with 0x0000):
                         ┌─────────────────────────────┐
 │ 0x0000_7FFF_FFFF_FF00 │  x: i32 = 42                │
 │                       │  [0x00][0x00][0x00][0x2A]   │
Low to high              ├─────────────────────────────┤
 │                       │                             │
 │ 0x0000_7FFF_FFFF_FF04 │  x_ref: &i32                │
 │                       │  [0x00][0x00][0x7F][0xFF]   │ Contains address: 0x0000_7FFF_FFFF_FF00
 │                       │  [0xFF][0xFF][0xFF][0x00]   │  (points to x)
 ↓                       └─────────────────────────────┘
```

**Key points about references:**

1. **References are pointers**: `&i32` is just an 8-byte address (on 64-bit systems)
2. **They point to existing data**: `x_ref` contains the address `0x0000_7FFF_FFFF_FF00` which is where `x` lives
3. **Borrow checker enforces rules at compile time**:
   - You can have many `&T` (immutable refs) OR one `&mut T` (mutable ref)
   - But NOT both at the same time
4. **References are always valid**: Compiler guarantees the pointed-to data exists

**Mutable references work the same way:**

```rust
let mut y: i32 = 100;
let y_mut_ref: &mut i32 = &mut y;
*y_mut_ref = 200;  // Dereference and modify
// y is now 200
```

```
Stack:
┌─────────────────────────────┐
│  y: i32 = 100               │  0x0000_7FFF_FFFF_FF10 (initially 100, then 200)
├─────────────────────────────┤
│  y_mut_ref: &mut i32        │  0x0000_7FFF_FFFF_FF14 (8 bytes)
│  [pointer to y]  ────────┐  │  Contains: 0x0000_7FFF_FFFF_FF10
└──────────────────────────┼──┘
                           │
    *y_mut_ref = 200  ─────┘  Writes through the pointer
```

**References vs Raw Pointers:**

- **References (`&T`, `&mut T`)**: Safe, borrow-checked, always valid
- **Raw pointers (`*const T`, `*mut T`)**: Unsafe, no checking, may be invalid

Let's see raw pointers next:

**Raw pointers come in two types, mirroring safe references:**

1. **`*const T`** - Read-only raw pointer, like `&T` but without safety guarantees
2. **`*mut T`** - Mutable raw pointer, like `&mut T` but without safety guarantees

**What safety guarantees are removed?**

With safe references (`&T`, `&mut T`), the compiler guarantees:

- ✅ Always points to valid, initialized data
- ✅ Properly aligned for the type
- ✅ Won't outlive the data it points to (lifetime checking)
- ✅ Exclusive access for `&mut T` (no aliasing mutable references)

With raw pointers (`*const T`, `*mut T`), **you** must ensure:

- ❌ May point to invalid/uninitialized data
- ❌ May be misaligned
- ❌ May outlive the data (dangling pointer)
- ❌ Multiple `*mut T` can exist to same location (you must prevent data races)

**Can raw pointers point to arbitrary addresses like in C?**

Yes! Unlike references, raw pointers can be created from arbitrary addresses:

```rust
// Point to GPU's VRAM framebuffer at a specific address
// Example: NVIDIA GeForce GTX 1650's prefetchable memory region (from lspci)
// Memory at c0000000 (64-bit, prefetchable) [size=256M]
let framebuffer: *mut u32 = 0xC000_0000 as *mut u32;

// Each pixel is a 32-bit color value (RGBA format)
// Pixel at position (x=100, y=50) in a 1920x1080 screen
let pixel_offset = (50 * 1920) + 100;  // y * width + x
let pixel_ptr = unsafe { framebuffer.add(pixel_offset) };

unsafe {
    // Write a red pixel: 0xFF0000FF (RGBA: Red=255, Green=0, Blue=0, Alpha=255)
    *pixel_ptr = 0xFF0000FF;  // Boom! A red dot appears on screen
}
```

**This example won't actually work** because the virtual address 0xC000_0000 in your process's
address space is **not mapped** to anything. While the GPU framebuffer exists at physical address
0xC000_0000, your process doesn't have a page table entry mapping the virtual address 0xC000_0000
to that physical location. Dereferencing it causes a page fault → segmentation fault.

> **Note:** Attempting to mmap this physical address region (e.g., via `/dev/mem`) will be
> **rejected by the kernel**. Modern Linux kernels have `CONFIG_STRICT_DEVMEM` enabled, which
> prevents mapping memory regions already claimed by device drivers. Since the GPU driver (nvidia,
> nouveau, amdgpu, etc.) has registered this PCI BAR region, direct userspace access is blocked.
> Additionally, the display server (Wayland/X11) has exclusive control via the DRM subsystem.
>
> This pattern works in: kernel drivers (which own the hardware), embedded systems without a
> display server, or bare-metal environments. This example demonstrates raw pointers' ability
> to reference arbitrary addresses - essential for hardware interaction and systems programming.

This is **extremely dangerous** but necessary for:

- Embedded systems (memory-mapped hardware)
- Operating system development
- Interfacing with C libraries
- Performance-critical code with manual memory management

**Key difference from safe references:**

```rust
let mut y: i32 = 42;

// ❌ Safe references: Can't have multiple mutable refs
let y_ref1 = &mut y;
let y_ref2 = &mut y;  // ERROR: cannot borrow as mutable more than once

// ✅ Raw pointers: Can have multiple mutable pointers
let y_ptr1: *mut i32 = &mut y;
let y_ptr2: *mut i32 = &mut y;
let y_ptr3: *mut i32 = &mut y;  // All OK! (but unsafe to use)

unsafe {
    *y_ptr1 = 100;  // Write 100 to y
    *y_ptr2 = 200;  // Overwrite with 200
    *y_ptr3 = 300;  // Overwrite with 300 (last write wins)

    // All three pointers point to the same location, so they all read 300
    println!("y_ptr1, y_ptr2, y_ptr3: {}, {}, {}", *y_ptr1, *y_ptr2, *y_ptr3);
    // Output: y_ptr1, y_ptr2, y_ptr3: 300, 300, 300
}

println!("y is now: {}", y);  // Prints: 300
```

**Why this is dangerous:** With multiple `*mut` pointers, you can create data races and undefined behavior - the compiler won't stop you!

**Pointers to heap data:**

So far, all our pointer examples pointed to stack data (like `&x` where `x` is on the stack) or arbitrary addresses. But how do pointers point to heap-allocated data? The answer: **allocation**.

To allocate memory on the heap, we use `std::alloc::alloc()` which returns a raw pointer to the allocated memory:

```rust
use std::alloc::{alloc, dealloc, Layout};

unsafe {
    // 1. Define the memory layout: we want space for 3 i32s (12 bytes)
    let layout = Layout::array::<i32>(3).unwrap();

    // 2. Allocate memory on the heap (alloc is unsafe!)
    let ptr: *mut i32 = alloc(layout) as *mut i32;

    // 3. Check if allocation succeeded (alloc returns null on failure)
    if ptr.is_null() {
        panic!("Allocation failed!");
    }

    // 4. Now ptr points to heap! We can write to it
    *ptr = 42;
    println!("Value at heap: {}", *ptr);  // Prints: 42

    // 5. Remember we allocated space for 3 i32s, so we can treat ptr like an array of 3
    *ptr.add(0) = 1;  // Write 1 at index 0 (first i32)
    *ptr.add(1) = 2;  // Write 2 at index 1 (second i32)
    *ptr.add(2) = 3;  // Write 3 at index 2 (third i32)

    // 6. What happens if we write beyond our allocation?
    // *ptr.add(3) = 4;  // ⚠️ UNDEFINED BEHAVIOR! We only allocated 3 i32s (indices 0-2)

    // 7. Read the values back
    println!("Heap data: {}, {}, {}", *ptr.add(0), *ptr.add(1), *ptr.add(2));
    // Output: Heap data: 1, 2, 3

    // 8. We MUST manually deallocate when done!
    dealloc(ptr as *mut u8, layout);
    // After dealloc, ptr is now a dangling pointer - using it is undefined behavior!
}
```

**Wait, primitives on the heap?**

Many people think primitives like `i32` always live on the stack. But that's not true! We just allocated three `i32`s **on the heap** using `alloc()`. The location of data (stack vs heap) isn't determined by the type - it's determined by **how you allocate it**:

- `let x: i32 = 42;` → `x` lives on **stack** (local variable)
- `alloc(Layout::new::<i32>())` → returns pointer to **heap** (manual allocation)

In our example, the three `i32` values (1, 2, 3) are sitting on the heap at addresses 0x5555_8000_0000, 0x5555_8000_0004, and 0x5555_8000_0008. They're heap-allocated primitives!

**What happens if we write beyond our allocation?**

Writing to `*ptr.add(3)` is **undefined behavior** - we only allocated 3 i32s (indices 0-2). Writing to index 3 is out-of-bounds and could:

- **Corrupt other heap data** - overwrite someone else's allocation
- **Trigger a segfault** - if `ptr+12` isn't in valid memory
- **Appear to work** - but corrupt memory silently
- **Cause mysterious bugs later** - when the corrupted data is used

**Important:** This won't cause a compilation error! Inside `unsafe` blocks, the compiler trusts you completely. It won't check bounds, validate pointers, or prevent undefined behavior. That's your responsibility now.

Unlike `Vec`, raw pointers don't do bounds checking! `Vec` would panic on `vec[3]` if `len=3`, but raw pointers trust you completely. This is why manual memory management is dangerous.

**Memory layout:**

After \*ptr.add(2) = 3, the heap looks like this:

```
Stack (0x7FFF_FFFF_FF00)              Heap (0x5555_8000_0000)
                                   (12 bytes total: 3 × 4-byte i32s)
    ┌─────────────────────┐         ┌─────┐
ptr │  0x5555_8000_0000  ─────────> │  1  │
    └─────────────────────┘         ┌─────┐
                                 +4 │  2  │
                                    ┌─────┐
                                 +8 │  3  │
                                    ┌─────┐
                                +12 │  4  │ *ptr.add(3) = 4 changed this, which is not owned by us!
                                    └─────┘
```

**Key points:**

1. **`alloc()` returns a pointer to heap memory** - the allocated bytes live on the heap
2. **Manual deallocation is required** - forgetting `dealloc()` causes a memory leak
3. **After `dealloc()`, the pointer is dangling** - using it causes undefined behavior
4. **This is extremely unsafe** - you must ensure:
   - The layout matches what you allocated
   - You don't use the pointer after dealloc
   - You don't call dealloc twice on the same pointer

**Smart pointers do this for you:**

Types like `Vec`, `String`, and `Box` internally use `alloc()` and `dealloc()`, but they:

- Call `alloc()` automatically when you create them
- Store the pointer in a struct on the stack
- Call `dealloc()` automatically in their `Drop` implementation
- Prevent you from using dangling pointers (via the borrow checker)

### The Heap

**What lives here:**

Since we now know about allocation, what lives here is anything that was allocated on the heap. Heap allocation needs two components:

- A raw pointer (to track where the allocation is)
- Allocation management (calling `alloc()` and `dealloc()`)

Rather than asking "which types live on the heap?", we should ask: **which Rust standard library types manage heap allocations internally?**

These types have a raw pointer field and call `alloc()`/`dealloc()`:

- `Box<T>` - the `T` value lives on heap
- `Vec<T>` - the array of `T` elements lives on heap
- `String` - the character data lives on heap
- `HashMap<K, V>` - the buckets and entries live on heap
- `Rc<T>` / `Arc<T>` - the `T` value lives on heap

**Important:** Types like `Option<T>`, `Result<T, E>`, `Cell<T>`, and `RefCell<T>` don't allocate on the heap by themselves. They're just wrappers around `T`:

- `Option<i32>` - entirely on stack (just an enum)
- `Option<Box<i32>>` - Box's pointer on stack, the `i32` on heap (because of `Box`, not `Option`)
- `RefCell<Vec<i32>>` - RefCell and Vec metadata on stack, Vec's array data on heap (because of `Vec`, not `RefCell`)

**How to know if a type allocates on the heap:**

Use "Go to Definition" in your IDE (or check the Rust standard library docs) to inspect the type's internal structure. If you see pointer fields like `*mut T`, that type manages heap allocations:

```rust
// Go to Definition on Option<T> shows:
pub enum Option<T> {
    None,
    Some(T), // ← Just contains T directly, no pointer!
}

// Go to Definition on Box<T> shows:
pub struct Box<T> {
    ptr: NonNull<T>,  // ← Let's Go to Definition on NonNull<T> to see what it is!
}
pub struct NonNull<T> {
    pointer: *const T, // ← Pointer, so Box<T> manages heap allocation
}

// Go to Definition on RefCell<T> shows:
pub struct RefCell<T> {
    borrow: Cell<BorrowFlag>,
    value: UnsafeCell<T>,  // ← Let's Go to Definition on UnsafeCell<T> to see what it is!
}
pub struct UnsafeCell<T> {
    value: T,  // ← Just contains T directly, no pointer!
}
```

**The rule:** If the type has a pointer field (`*mut T`, `*const T`, `NonNull<T>`), it manages heap allocation. Otherwise, it's just a wrapper that lives on the stack.

**Characteristics:**

- **Manual management**: You allocate/deallocate (Rust does this for you via `Drop`)
- **Slower allocation**: Requires finding a free block (complex algorithms)
- **Large size**: Typically gigabytes (depends on available RAM)

**Example:**

```rust
fn example() {
    // Stack: 24 bytes (Vec metadata)
    // Heap: 400 bytes (100 * 4-byte integers)
    let v = vec![0; 100];

    // Stack: 24 bytes (String metadata)
    // Heap: Variable (depends on string length)
    let s = String::from("hello");

    // Stack: 8 bytes (Box pointer)
    // Heap: 4 bytes (i32)
    let b = Box::new(42);
}  // Drop is called, heap memory is freed
```

**Heap allocation is expensive:**

```rust
// Allocates once, then grows as needed (reallocating)
let mut v = Vec::new();
for i in 0..1000 {
    v.push(i);  // Might allocate/reallocate
}

// Pre-allocate: only allocates once
let mut v = Vec::with_capacity(1000);
for i in 0..1000 {
    v.push(i);  // No allocation needed
}
```

### Static Data (DATA Segment)

**What lives here:**

- `static` variables
- `const` values (inlined, but literals live here)
- String literals (`"hello"`)
- Binary data embedded at compile time

**Characteristics:**

- **Loaded at program start**: Burned into the executable
- **Lives forever**: Never deallocated (program lifetime)
- **Fixed size**: Known at compile time
- **Read-only or read-write**: Depends on whether it's `static` or `static mut`

**Example:**

```rust
static GREETING: &str = "Hello, world!";  // DATA segment
const MAX: i32 = 100;                     // Inlined (no memory allocated)

fn main() {
    println!("{}", GREETING);  // Uses data from DATA segment
    let x = MAX;               // Constant inlined: let x = 100;
}
```

**String literals are special:**

```rust
let s1 = "hello";  // Points to DATA segment
let s2 = "hello";  // Points to SAME location in DATA segment!
assert_eq!(s1.as_ptr(), s2.as_ptr());  // Same address!

let s3 = String::from("hello");  // Allocates on heap (different address)
```

## Visualizing Types

Let's see where different types memory layout:

### Simple Types (Copy)

```rust
let x: i32 = 42;
let y: bool = true;
let z: f64 = 3.14;

Stack:
┌──────────────┐
│ x: i32 = 42  │  4 bytes
│ y: bool = 1  │  1 byte (+ padding)
│ z: f64 = ... │  8 bytes
└──────────────┘

Heap: (nothing)
```

### Arrays (Fixed Size)

```rust
let arr: [i32; 5] = [1, 2, 3, 4, 5];

Stack:
┌──────────────────────────┐
│ arr: [i32; 5]            │
│   [1][2][3][4][5]        │  20 bytes
└──────────────────────────┘

Heap: (nothing)
```

### String

```rust
let s = String::from("hello");

Stack:                      Heap:
┌──────────────────┐       ┌──────────────────┐
│ s: String        │       │                  │
│   ptr ──────────────────>│ [h][e][l][l][o]  │
│   len: 5         │       │  5 bytes         │
│   cap: 5         │       │                  │
└──────────────────┘       └──────────────────┘
    24 bytes                   5 bytes (+ capacity)
```

### Vec

```rust
let v = vec![1, 2, 3];

Stack:                      Heap:
┌──────────────────┐       ┌──────────────────┐
│ v: Vec<i32>      │       │                  │
│   ptr ──────────────────>│ [1][2][3]        │
│   len: 3         │       │  12 bytes        │
│   cap: 3         │       │                  │
└──────────────────┘       └──────────────────┘
    24 bytes                   12 bytes
```

### Box

```rust
let b = Box::new(42);

Stack:                      Heap:
┌──────────────────┐       ┌──────┐
│ b: Box<i32>      │       │      │
│   ptr ──────────────────>│  42  │
└──────────────────┘       │      │
    8 bytes                └──────┘
                            4 bytes
```

### Nested Types

```rust
let v: Vec<String> = vec![
    String::from("hello"),
    String::from("world"),
];

Stack:                          Heap:
┌────────────────────┐         ┌─────────────────────────────────────────┐
│ v: Vec<String>     │         │  String 0:                              │
│   ptr ─────────────────────> │    ├─ ptr  ──┐                          │
│   len: 2           │         │    ├─ len: 5 │  (24 bytes)              │
│   cap: 2           │         │    └─ cap: 5 │                          │
└────────────────────┘         │              │                          │
                               │  String 1:   │                          │
                               │    ├─ ptr  ──┼──┐                       │
                               │    ├─ len: 5 │  │  (24 bytes)           │
                               │    └─ cap: 5 │  │                       │
                               │              ↓  │                       │
                               │   "hello"  [h][e│[l][l][o] (5 bytes)    │
                               │                 ↓                       │
                               │       "world"  [w][o][r][l][d] (5 bytes)│
                               └─────────────────────────────────────────┘

- Stack: 24 bytes (Vec metadata)
- Heap: 48 bytes (2 × String metadata: 2 × 24 bytes) + 10 bytes (string data)
- Total heap: 58 bytes
```

**Three levels of indirection!**

1. `v` points to array of `String`s
2. Each `String` points to its character data
3. All on the heap

## Common Misconceptions

### Misconception #1: "Vec allocates on the stack"

```rust
let v = Vec::new();
```

**Wrong mental model:**

```
Stack:
┌────────────────────┐
│ v: Vec<i32>        │
│   [data goes here] │  ← NO! Data doesn't live here
└────────────────────┘
```

**Correct mental model:**

```
Stack:                    Heap:
┌──────────────┐         ┌──────────┐
│ v: Vec<i32>  │         │          │
│   ptr ──────────────>  │ (data)   │  ← Data lives here!
│   len: 0     │         │          │
│   cap: 0     │         └──────────┘
└──────────────┘
```

### Misconception #2: "String is just text"

```rust
let s = String::from("hello");
```

**Wrong:**
"s is the text 'hello'"

**Correct:**
"s is a struct containing a pointer to the text 'hello' on the heap"

```
Stack: s = { ptr: 0x1000, len: 5, cap: 5 }  (24 bytes)
Heap:  0x1000 = "hello"                     (5 bytes)
```

### Misconception #3: "Box makes things bigger"

```rust
let x = 42;           // 4 bytes
let b = Box::new(42); // How many bytes?
```

**Answer:** `b` is 8 bytes (just a pointer), but total memory usage is 12 bytes (8 + 4).

**However:** Boxing can actually **save stack space** for large types:

```rust
let huge = [0u8; 1_000_000];        // 1 MB on stack! Dangerous!
let boxed = Box::new([0u8; 1_000_000]); // 8 bytes on stack, 1 MB on heap
```

### Misconception #4: "All heap allocations are slow"

Not all heap operations allocate:

```rust
let mut v = Vec::with_capacity(100);  // ✅ One allocation

for i in 0..50 {
    v.push(i);  // ✅ No allocation - within capacity
}

v.push(51);  // ✅ Still no allocation
v.push(52);  // ✅ Still no allocation
// ... up to 100 elements, still no allocation

v.push(101);  // ❌ NOW we reallocate (capacity exceeded)
```

Pre-allocating capacity is a common optimization!

## Performance Implications

### Stack Operations (Fast)

```rust
fn stack_test() {
    let x = 42;        // ~1 CPU cycle (just move stack pointer)
    let y = x;         // ~1 CPU cycle (copy 4 bytes)
}  // ~1 CPU cycle (move stack pointer back)
```

**Cost:** ~3 CPU cycles

### Heap Operations (Slow)

```rust
fn heap_test() {
    let x = Box::new(42);  // ~100 CPU cycles (call allocator)
    let y = x;             // ~1 CPU cycle (copy 8-byte pointer)
}  // ~100 CPU cycles (call deallocator)
```

**Cost:** ~200 CPU cycles

**100x slower!** But remember:

- This is microseconds, not seconds
- Sometimes you need the heap (dynamic size, large data, shared ownership)
- The real cost is in **many allocations**, not just one

### Optimization Tips

1. **Pre-allocate collections:**

```rust
// Bad: multiple allocations
let mut v = Vec::new();
for i in 0..1000 { v.push(i); }

// Good: one allocation
let mut v = Vec::with_capacity(1000);
for i in 0..1000 { v.push(i); }
```

2. **Use `&str` instead of `String` when possible:**

```rust
// Bad: allocates on heap
fn greet(name: String) {
    println!("Hello, {}", name);
}

// Good: no allocation
fn greet(name: &str) {
    println!("Hello, {}", name);
}
```

3. **Use `[T; N]` instead of `Vec<T>` for fixed-size data:**

```rust
// Bad: heap allocation
let v = vec![0; 10];

// Good: stack allocation
let arr = [0; 10];
```

4. **Avoid cloning when borrowing works:**

```rust
// Bad: clones the string (heap allocation)
fn process(s: String) {
    println!("{}", s);
}
let s = String::from("hello");
process(s.clone());

// Good: borrows (no allocation)
fn process(s: &str) {
    println!("{}", s);
}
process(&s);
```

## Memory Leaks

Rust prevents many memory leaks, but they're still possible:

### Safe Memory Leak (Reference Cycles)

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Node {
    next: Option<Rc<RefCell<Node>>>,
}

let a = Rc::new(RefCell::new(Node { next: None }));
let b = Rc::new(RefCell::new(Node { next: Some(Rc::clone(&a)) }));
a.borrow_mut().next = Some(Rc::clone(&b));

// Memory leak! a and b reference each other
// Neither will ever be dropped (reference count never reaches 0)
```

**Solution:** Use `Weak` references to break cycles.

### Intentional Memory Leak

```rust
let s = String::from("hello");
let leaked: &'static str = Box::leak(Box::new(s));
// s is now leaked - memory will never be freed
// But we got a 'static reference!
```

**Use case:** Creating `'static` data at runtime (rare).

## Key Takeaways

1. **Stack is automatic** - variables disappear when out of scope
2. **Heap is manual** - you allocate/deallocate (Rust automates via `Drop`)
3. **Stack is fast** - just move a pointer
4. **Heap is flexible** - dynamic size, outlives scope
5. **String/Vec/Box are smart pointers** - metadata on stack, data on heap
6. **Static data lives forever** - loaded at program start
7. **Use stack by default** - only heap allocate when necessary
8. **Pre-allocate when possible** - avoid repeated reallocations

## Further Reading

- [cheats.rs/#memory-layout](https://cheats.rs/#memory-layout) - Visual memory layouts for Rust types
- **The Rustonomicon**: Memory layout and representation
- **Rust Performance Book**: Memory allocation strategies
- **Operating Systems textbooks**: Virtual memory, process address space

---
