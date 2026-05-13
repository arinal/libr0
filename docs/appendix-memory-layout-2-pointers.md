# Raw Pointers & Unsafe

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

```bob
STACK
                         +-----------------------+
"0x7FFF_FFFF_FF00"     x |  42                   |<--+
                         +-----------------------+   |
"0x7FFF_FFFF_FF04  x_ref"| "0x7FFF_FFFF_FF00" *--+---+
                         +-----------------------+
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
```

```bob
       STACK
  y               "y_mut_ref"
+------+       +---------------+
| 100  |<------|  Address of y |
+------+       +---------------+
```

```rust
*y_mut_ref = 200;
```

Dereferencing `y_mut_ref` modifies the value of `y` in place — `y_mut_ref` itself still holds the same address, the address of `y`.

```bob
       STACK
  y               "y_mut_ref"
+------+       +---------------+
| 200  |<------|  Address of y |
+------+       +---------------+
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

```bob
"Stack (0x7FFF_FFFF_FF00)"              "Heap (0x5555_8000_0000)"
                                   "(12 bytes total: 3 × 4-byte i32s)"
    +---------------------+         +-----+
ptr |  "0x5555_8000_0000" +-------> |  1  |
    +---------------------+         +-----+
                                    |  2  | +4
                                    +-----+
Last element of our allocation  --> |  3  | +8
                                    +-----+
Beyond our allocation  -----------> |  4  | +12
                                    +-----+
```

**Key points:**

1. **`alloc()` returns a pointer to heap memory** - the allocated bytes live on the heap
1. **Writing beyond the allocation is undefined behavior** - `ptr.add(3)` points past the 3 bytes we allocated, and writing there silently corrupts memory that may belong to another allocation
1. **Manual deallocation is required** - forgetting `dealloc()` causes a memory leak
1. **After `dealloc()`, the pointer is dangling** - using it causes undefined behavior
1. **This is extremely unsafe** - you must ensure:
   - The layout matches what you allocated
   - You don't use the pointer after dealloc
   - You don't call dealloc twice on the same pointer

**Smart pointers do this for you:**

Types like `Vec`, `String`, and `Box` internally use `alloc()` and `dealloc()`, but they:

- Call `alloc()` automatically when you create them
- Store the pointer in a struct on the stack
- Call `dealloc()` automatically in their `Drop` implementation
- Prevent you from using dangling pointers (via the borrow checker)

