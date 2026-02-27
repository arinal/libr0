---
marp: true
theme: default
paginate: true
---

# Memory Layout
## Where Your Data Lives in Rust

A beginner-friendly visual tour of the stack, heap, and static data.

---

## The Simple Program

We will trace where everything lives in memory for this program:

```rust
static GREETING: &str = "Hello";

fn main() {
    let x = 42;
    let s = String::from("world");
    let v = vec![1, 2, 3, 4, 5];
    let arr = [10, 20, 30, 40, 50];
    let doubled = process_data(x, &s);
}

fn process_data(param_num: i32, param_text: &String) -> i32 {
    let result = param_num * 2;
    result
}
```

---

## Process Memory Layout

When your Rust program runs, memory is organized into distinct regions:

```
High Addresses
┌─────────────────────────────────────┐
│  STACK  (grows ↓)                   │  local variables, function frames
├─────────────────────────────────────┤
│         (unused space)              │
├─────────────────────────────────────┤
│  HEAP   (grows ↑)                   │  Box, Vec, String data
├─────────────────────────────────────┤
│  BSS    (zeroed static data)        │  static mut, uninitialized statics
├─────────────────────────────────────┤
│  DATA   (initialized static data)   │  static, string literals
├─────────────────────────────────────┤
│  TEXT   (compiled code)             │  your functions
└─────────────────────────────────────┘
Low Addresses
```

> **Key insight:** Stack and heap grow toward each other.

---

## Step 1: Static Data Loads First

Before `main()` runs, the OS loads static data into the DATA segment:

```
DATA segment:
  0x5000: GREETING = "Hello"
          ├─ ptr:  0x5000
          ├─ len:  5
          └─ "Hello\0"

TEXT segment:
  0x1000: fn main() { ... }
  0x2000: fn process_data() { ... }
```

**Why BSS exists:**
- `static BUFFER: [u8; 1_000_000] = [0; 1_000_000];`
- BSS: just stores "give me 1 MB of zeros" (~16 bytes in the file!)
- Data segment: would store all 1 million bytes in the binary
- **Same memory at runtime, much smaller executable!**

---

## Step 2: main() Stack Frame

After all local variables are initialized:

```
STACK:
┌────────────────────────────────┐
│  [Return address]              │
│  x: i32 = 42                  │  4 bytes
│  s: String                    │
│    ├─ ptr: 0x8000 ─────────┐  │  24 bytes (metadata only!)
│    ├─ len: 5               │  │
│    └─ cap: 5               │  │
│  v: Vec<i32>               │  │
│    ├─ ptr: 0x8100 ──────┐  │  │  24 bytes (metadata only!)
│    ├─ len: 5            │  │  │
│    └─ cap: 5            │  │  │
│  arr: [i32; 5]          │  │  │  20 bytes (all data on stack!)
│    [10][20][30][40][50] │  │  │
└─────────────────────────┼──┼──┘
HEAP:                     ↓  ↓
  0x8000: [w][o][r][l][d]       s's actual data
  0x8100: [1][2][3][4][5]       v's actual data
```

---

## Local Variables: The Simple Rule

> **The Rule: Everything declared with `let` lives on the stack.**

The _type_ does not matter — the _scope_ does.

```rust
let x: i32 = 5;          // stack (primitive)
let arr: [i32; 3] = ...; // stack (array, all data inline)
let result = Ok(42);      // stack (enum including its data)
let ref_x: &i32 = &x;    // stack (pointer itself, points to x on stack)
```

But what about `String`, `Vec`, `Box`? They are just structs:

```rust
let v = Vec::new();   // 24 bytes on stack (ptr + len + cap)
v.push(1);            // NOW heap is allocated! ptr points to heap.
```

**Key insight:** `Vec`'s metadata (ptr/len/cap) is always on the stack.
Only the _element data_ it manages lives on the heap.

---

## Step 3: Calling process_data()

Arguments are passed via **CPU registers** — not the stack!

```
CPU REGISTERS:
  EDI = 42              ← copy of x (pass by value)
  RSI = 0x7FFF...       ← pointer to s (pass by reference)

STACK:
┌──────────────────────────────┐  main's frame
│  x: i32 = 42                 │
│  s: String (ptr → heap)      │
│  ...                         │
├──────────────────────────────┤  process_data's frame (new!)
│  [Return address to main]    │
│  [Saved RBP]                 │
│  result: i32                 │  4 bytes, locally computed
└──────────────────────────────┘
```

> **param_num and param_text are NOT on the stack** — they ARE the registers!

---

## Step 3: What's Happening in the CPU

```asm
; Calling process_data(x, &s):
mov  edi, DWORD PTR [rbp-4]  ; Load x (42) into EDI
lea  rsi, [rbp-32]           ; Load address of s into RSI
call process_data            ; Push return address, jump

; Inside process_data:
push rbp                     ; Save caller's base pointer
mov  rbp, rsp                ; Set up our base pointer
sub  rsp, 16                 ; Allocate space for locals

mov  eax, edi                ; param_num (42) into EAX
shl  eax, 1                  ; times 2 = 84
mov  DWORD PTR [rbp-4], eax  ; store result

mov  eax, DWORD PTR [rbp-4]  ; return value into RAX
add  rsp, 16
pop  rbp
ret
```

**x86-64 argument registers:** RDI, RSI, RDX, RCX, R8, R9
Return value lives in RAX.

---

## Step 4: Return and Cleanup

When `process_data()` returns:
1. Return value (84) is copied from RAX to `doubled` in main's frame
2. process_data's stack frame is **gone**
3. Heap is untouched

```
STACK (after return):
┌──────────────────────────────┐
│  main's frame only           │
│  ...                         │
│  doubled: i32 = 84           │  return value copied here
└──────────────────────────────┘

HEAP: unchanged
  0x8000: [w][o][r][l][d]
  0x8100: [1][2][3][4][5]
```

---

## Step 5: main() Ends — Drop Runs

When `s` and `v` go out of scope, Rust calls their `Drop` implementations:

```
Before:
  Stack: s (ptr → 0x8000), v (ptr → 0x8100)
  Heap:  0x8000: "world", 0x8100: [1,2,3,4,5]

After drop:
  Stack: (popped)
  Heap:  0x8000: (freed), 0x8100: (freed)
  DATA:  GREETING still lives here forever
```

**You do not write `free()` in Rust.**
Drop is called automatically at end of scope. No leaks, no double-frees.

---

## References: Just Pointers

References are just **memory addresses** — safe pointers enforced at compile time.

```rust
let x: i32 = 42;
let x_ref: &i32 = &x;
```

```
Stack:
┌──────────────────────────────┐
│  x: i32 = 42                 │  0x7FFF_FFFF_FF00  (4 bytes)
├──────────────────────────────┤
│  x_ref: &i32                 │  0x7FFF_FFFF_FF04  (8 bytes)
│  contains: 0x7FFF_FFFF_FF00  │  ← address of x
└──────────────────────────────┘
```

**Borrow rules (compile-time enforced):**
- Many `&T` (immutable), OR
- One `&mut T` (mutable)
- Never both at the same time

---

## Raw Pointers

`*const T` / `*mut T` — like references, but **you** are responsible for safety.

```rust
let mut y: i32 = 42;
let ptr1: *mut i32 = &mut y;
let ptr2: *mut i32 = &mut y; // OK to create, dangerous to use!

unsafe {
    *ptr1 = 100;
    *ptr2 = 200; // both point to same location
    println!("{}", *ptr1); // 200
}
```

| Guarantee | `&T` | `*const T` |
|-----------|------|------------|
| Always valid | yes | no |
| Properly aligned | yes | no |
| No dangling | yes | no |
| No aliased mut | yes | no |

---

## Heap Allocation: The Basics

Heap memory is requested at runtime via `alloc()`:

```rust
use std::alloc::{alloc, dealloc, Layout};

unsafe {
    let layout = Layout::array::<i32>(3).unwrap();
    let ptr: *mut i32 = alloc(layout) as *mut i32;

    *ptr.add(0) = 1;
    *ptr.add(1) = 2;
    *ptr.add(2) = 3;

    dealloc(ptr as *mut u8, layout); // MUST do this manually!
}
```

```
Stack: ptr (8 bytes)  ──>  Heap: [1][2][3] (12 bytes)
```

`Vec`, `Box`, `String` wrap this pattern — including the `dealloc` via `Drop`.

---

## Type Memory Layouts

**Simple types (Copy):**
```
let x: i32 = 42;         Stack: [42]           4 bytes, no heap
let arr: [i32; 5] = ...; Stack: [1][2][3][4][5] 20 bytes, no heap
```

**Box:**
```
let b = Box::new(42);
  Stack: ptr (8 bytes)  →  Heap: 42 (4 bytes)
```

**String / Vec:**
```
let s = String::from("hello");
  Stack: { ptr, len: 5, cap: 5 }  (24 bytes)
  Heap:  [h][e][l][l][o]          (5 bytes)
```

**Vec\<String\> — three levels of indirection:**
```
Stack: ptr → Heap: [String{ptr,len,cap}, String{ptr,len,cap}]
                          ↓                      ↓
                      "hello"                "world"
```

---

## Which Types Allocate on the Heap?

**Check for pointer fields** in the type definition:

```rust
// Box<T> has a pointer field → allocates heap!
pub struct Box<T> {
    ptr: NonNull<T>,  // *const T inside
}

// Option<T> has no pointer → does NOT allocate by itself
pub enum Option<T> {
    None,
    Some(T),  // just contains T directly
}

// RefCell<T> has no pointer → does NOT allocate by itself
pub struct RefCell<T> {
    borrow: Cell<BorrowFlag>,
    value: UnsafeCell<T>,  // just contains T directly
}
```

**The rule:** If a type has `*mut T`, `*const T`, or `NonNull<T>`, it manages heap memory.

---

## Common Misconceptions

**"Primitives always live on the stack"**
Wrong! Location depends on how you allocate:
```rust
let x: i32 = 42;             // stack (local variable)
let p = Box::new(42i32);     // i32 on HEAP (boxed)
```

**"String is just text"**
String is a struct: `{ ptr, len, cap }`.
The struct is on the stack. The text is on the heap.

**"Box makes things bigger"**
`Box::new(42)` → 8 bytes on stack, 4 bytes on heap.
Boxing can **save** stack space for large types:
```rust
let big = Box::new([0u8; 1_000_000]); // 8 bytes stack, 1 MB heap
```

---

## Stack vs Heap Performance

| | Stack | Heap |
|--|-------|------|
| Allocation | ~1 CPU cycle | ~100 CPU cycles |
| Deallocation | ~1 CPU cycle | ~100 CPU cycles |
| Access | Fast (cache friendly) | Slower (pointer indirection) |
| Max Size | 2–8 MB | GBs |
| Management | Automatic | Automatic via `Drop` |

**Pre-allocate to avoid repeated heap allocations:**

```rust
// Potentially many reallocations:
let mut v = Vec::new();
for i in 0..1000 { v.push(i); }

// Exactly one allocation:
let mut v = Vec::with_capacity(1000);
for i in 0..1000 { v.push(i); }
```

---

## Optimization Tips

**Use `&str` instead of `String` when you do not need ownership:**
```rust
fn greet(name: &str) { println!("Hello, {}", name); }  // no allocation
```

**Use `[T; N]` instead of `Vec<T>` for fixed-size data:**
```rust
let arr = [0; 10];   // stack, no allocation
let v = vec![0; 10]; // heap, allocation
```

**Avoid `clone()` when borrowing works:**
```rust
fn process(s: &str) { ... }
process(&my_string);  // borrow, no copy
```

**Use `with_capacity` for collections you will fill:**
```rust
let mut map = HashMap::with_capacity(100);
```

---

## Key Takeaways

1. **`let` → stack.** All local variables live on the stack regardless of type.
2. **Heap via smart pointers.** `Vec`, `String`, `Box` manage heap for you.
3. **References are just pointers.** 8 bytes on 64-bit, always.
4. **Drop replaces free().** Rust cleans up heap memory automatically.
5. **Stack is fast; heap is flexible.** Use stack by default.
6. **Static data lives forever.** Loaded once at program start.
7. **Pre-allocate.** Avoid repeated reallocations with `with_capacity`.

---

## Further Reading

- [cheats.rs/#memory-layout](https://cheats.rs/#memory-layout) — Visual memory layouts for every Rust type
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/) — Memory layout and unsafe Rust
- [Rust Performance Book](https://nnethercote.github.io/perf-book/) — Allocation strategies
- [under-the-hood](https://github.com/arinal/under-the-hood) — Low-level concepts (syscalls, blocking I/O)

---

# Thanks! 🦀

> *"De-abstract the abstractions. Rust is not magic — it is just code all the way down."*
