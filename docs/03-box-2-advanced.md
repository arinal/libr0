# Advanced Methods & Deref Coercion

### leak - Intentionally Leak Memory

Sometimes you want to keep data alive forever without deallocation:

```rust
impl<T> Box0<T> {
    fn leak(self) -> &'static mut T {
        let ptr = self.ptr;
        std::mem::forget(self);  // Don't run Drop
        unsafe { &mut *ptr }
    }
}
```

**Example:**

```rust
let boxed = Box0::new(42);
let leaked: &'static mut i32 = boxed.leak();
*leaked = 100;  // Can mutate forever
// Memory is never freed!
```

**Is this safe?**

Yes! `leak()` is **safe** (not marked `unsafe`) because:

- It doesn't cause undefined behavior
- The returned reference has `'static` lifetime, valid for the entire program
- The heap memory stays allocated and accessible through the reference

**What if we don't save the returned reference?**

```rust
fn leak_and_lose() {
    let boxed = Box0::new(42);
    let leaked: &'static mut i32 = boxed.leak();  // Get the reference
    // leaked is a local variable that will be destroyed when function ends
    // But the heap memory it points to? Still there!
}

leak_and_lose();  // Function ends, local variable 'leaked' is gone
// The i32 is still on the heap at some address, taking up 4 bytes
// But we have no way to access it anymore - the reference is lost!
```

This is still **safe** (no UB), but it's a **useless memory leak**:

- The heap memory is leaked (never freed)
- The local variable `leaked` (just a pointer on the stack) is destroyed
- But we can't access the heap data because we lost the reference
- The data sits in memory for the rest of the program, wasting space

When `leaked` goes out of scope, only the **reference** (a stack pointer) is removed. The **heap data** remains forever - that's the whole point of `leak()`.

Use cases: global state, thread-local storage, or when interfacing with C code that expects static lifetime.

### into_raw and from_raw - Raw Pointer Conversion

Convert to/from raw pointers for FFI or manual memory management:

```rust
impl<T> Box0<T> {
    fn into_raw(self) -> *mut T {
        let ptr = self.ptr;
        std::mem::forget(self);  // Don't run Drop
        ptr
    }

    unsafe fn from_raw(ptr: *mut T) -> Box0<T> {
        Box0 { ptr }
    }
}
```

**Example:**

```rust
extern "C" { fn c_process_data(ptr: *mut String); }

let boxed = Box0::new(String::from("hello"));
let ptr = Box0::into_raw(boxed);
unsafe { c_process_data(ptr); }  // Pass to C

// from_raw is UNSAFE - you must guarantee the pointer came from into_raw
let restored = unsafe { Box0::from_raw(ptr) };  // Get it back
```

**Warning:** `from_raw` is `unsafe` because:

- The pointer must have come from `into_raw`
- You must not use it after calling `from_raw` (double-free!)
- The pointer must not be null

Compare to dereferencing and moving:

```rust
let boxed = Box0::new(String::from("hello"));
let s = *boxed;  // Move out of box
// ERROR: Can't move out of `*boxed` because Box implements Deref but not DerefMove
```

This doesn't work with the real `Box` either - you need `Box::into_inner()` (or just let the box drop if you want both gone).

## Deref Coercion

One of Rust's nicest features. When you have `&Box0<T>`, it can automatically become `&T`:

```rust
fn print_len(s: &str) {
    println!("Length: {}", s.len());
}

let boxed = Box0::new(String::from("hello"));
print_len(&boxed);  // &Box0<String> -> &String -> &str
```

**How does this work?**

Deref coercion is a **special compiler feature** that only works with the `Deref` trait. The compiler automatically inserts deref calls to make types match:

1. You pass `&boxed`, which is `&Box0<String>`
2. Function expects `&str`
3. Compiler tries: "Can I turn `&Box0<String>` into `&str`?"
4. First deref: `&Box0<String>` → calls `deref()` → `&String`
5. Second deref: `&String` → calls `deref()` → `&str` ✅ Match!

The compiler chains `Deref` implementations automatically. This **only works** with:

- The `Deref` trait (for immutable references)
- The `DerefMut` trait (for mutable references)

You can't create your own trait with this behavior - it's built into the compiler specifically for `Deref`/`DerefMut`.

## Vec and String: Box with Extra Metadata

`Box` isn't the only type that uses heap allocation. `Vec` and `String` do too - they're essentially "fat" pointers with extra fields:

```rust
// Simplified Vec definition
struct Vec<T> {
    ptr: *mut T,   // Pointer to heap data (like Box)
    len: usize,    // Number of elements currently stored
    cap: usize,    // Total allocated capacity
}

// Simplified String definition
struct String {
    vec: Vec<u8>,  // String is just a Vec<u8> with UTF-8 guarantee
}

// Which means String is really:
struct String {
    ptr: *mut u8,  // Pointer to heap-allocated bytes
    len: usize,    // Length in bytes
    cap: usize,    // Capacity in bytes
}
```

Compare to Box:

```rust
struct Box<T> {
    ptr: *mut T,   // Just the pointer, nothing else
}
```

| Type     | Stack size                 | Heap data   |
| -------- | -------------------------- | ----------- |
| `Box<T>` | 8 bytes (ptr)              | `T`         |
| `Vec<T>` | 24 bytes (ptr + len + cap) | `[T; cap]`  |
| `String` | 24 bytes (ptr + len + cap) | `[u8; cap]` |

All three:

- Allocate on the heap
- Implement `Deref` for ergonomic access
- Implement `Drop` to free memory automatically

## Exercises

See the full code in [`src/box.rs`](./src/box.rs) for the complete implementation of `Option0` with all methods.
Also, see the exercises in [01_box.rs](./examples/01_box.rs)

**Complete solutions:** Switch to the `answers` branch with `git checkout answers` to see completed exercises

## Key Takeaways

1. **Box is just a pointer** - Single pointer to heap-allocated data
2. **Deref enables ergonomics** - Use `*b` to access inner value
3. **Drop ensures cleanup** - Memory freed when Box goes out of scope
4. **Deref coercion is magic** - `&Box<T>` automatically becomes `&T`
5. **Use for recursive types** - Break infinite size with indirection