# Building Our Own Box

We can't truly replicate `Box` without compiler magic, but we can understand its core:

```rust
use std::alloc::{alloc, dealloc, Layout};
use std::ptr;

struct Box0<T> {
    ptr: *mut T,
}
```

### new - Allocate and Store

```rust
impl<T> Box0<T> {
    fn new(value: T) -> Box0<T> {
        unsafe {
            // 1. Calculate memory layout for T
            let layout = Layout::new::<T>();

            // 2. Allocate memory
            let ptr = alloc(layout) as *mut T;

            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            // 3. Write value to allocated memory
            ptr::write(ptr, value);

            Box0 { ptr }
        }
    }
}
```

### The Deref Trait

`Deref` lets us use `*box_value` to get the inner value:

```rust
use std::ops::Deref;

impl<T> Deref for Box0<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}
```

With `Deref`, this works:

```rust
let b = Box0::new(5);
*b  // 5

// Can we move out with *b?
let c = *b;  // ✅ Works! i32 is Copy, so this copies the value
c  // 5

// What about non-Copy types?
let s = Box0::new(String::from("hello"));

// This works fine - gets a reference:
let borrowed = *s;  // borrowed: &String

// But if you explicitly ask for ownership, it fails:
// let owned: String = *s;  // ❌ ERROR: cannot move out of `*s`
```

What's happening?

`*s` calls `deref()` which returns `&String`. What happens next depends on what you're trying to bind:

```rust
let borrowed = *s;            // ✅ Infers &String, works
let borrowed: &String = *s;   // ✅ Explicitly &String, works
let owned: String = *s;       // ❌ Asks for String (ownership), fails!
```

When you write `let owned: String = *s;`:

- `*s` gives `&String` (a reference)
- You're asking for `String` (ownership, not a reference)
- Compiler tries: "Can I move `String` out of `&String`?"
- Answer: No! Moving from a reference is not allowed → **error!**

The key insight:

- **Copy types** (`i32`): `let c = *b;` copies the value from `&i32` → works!
- **Non-Copy types, asking for reference** (`&String`): `let x = *s;` or `let x: &String = *s;` → works!
- **Non-Copy types, asking for ownership** (`String`): `let x: String = *s;` → fails!

To get ownership of non-Copy types, use `into_inner()` to consume the box.

### DerefMut for Mutable Access

```rust
use std::ops::DerefMut;

impl<T> DerefMut for Box0<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}
```

Now we can mutate:

```rust
let mut b = Box0::new(5);
*b = 10;
```

### Drop - Clean Up Memory

The whole point of smart pointers: automatic cleanup.

```rust
impl<T> Drop for Box0<T> {
    fn drop(&mut self) {
        unsafe {
            // 1. Call destructor on the value
            ptr::drop_in_place(self.ptr);

            // 2. Deallocate the memory
            let layout = Layout::new::<T>();
            dealloc(self.ptr as *mut u8, layout);
        }
    }
}
```

**When is Drop called?**

```rust
// 1. End of scope
{
    let b = Box0::new(String::from("hello"));
    println!("Using box: {}", *b);
}  // Drop called here - memory freed automatically

// 2. Explicit drop
let b = Box0::new(42);
drop(b);  // Drop called immediately
// b is no longer valid

// 3. Reassignment
let mut b = Box0::new(10);
b = Box0::new(20);  // Drop called on old Box0(10), then new one assigned

// 4. NOT called when using into_inner, leak, or into_raw
let b = Box0::new(5);
let value = b.into_inner();  // Drop NOT called - we handled cleanup manually
```

### into_inner - Extract Value Without Drop

Sometimes you want to move the value out of the box and take ownership, deallocating the box itself but keeping the value:

```rust
impl<T> Box0<T> {
    fn into_inner(self) -> T {
        unsafe {
            // 1. Read the value from heap into stack variable
            //    ptr::read copies/moves T from *self.ptr (heap) to local variable
            let value = ptr::read(self.ptr);

            // 2. Deallocate the heap memory (but value is now on stack!)
            let layout = Layout::new::<T>();
            dealloc(self.ptr as *mut u8, layout);

            // 3. Prevent Drop from running
            std::mem::forget(self);

            // 4. Return the extracted value (moves to caller)
            value
        }
    }
}
```

**Why `mem::forget(self)`?**

Without it, `Drop::drop` would run when `self` goes out of scope at the end of this function. But we already:

1. Read the value with `ptr::read`
2. Deallocated the memory with `dealloc`

If `Drop` ran, it would:

1. Try to drop the value that's no longer there (use-after-free!)
2. Try to deallocate already-freed memory (double-free!)

Both are undefined behavior. `mem::forget` tells the compiler "don't run Drop on this value."

**Example:**

```rust
let boxed = Box0::new(String::from("hello"));
let s = boxed.into_inner();  // Extract String, deallocate Box
s  // "hello" - we now own the String directly
// boxed is consumed, but Drop wasn't called
```

**Wait, is the String on the stack now?**

Not quite! Remember that `String` itself is just a struct:

```rust
struct String {
    ptr: *mut u8,  // Pointer to heap data
    len: usize,
    cap: usize,
}
```

After `into_inner()`:

- The `String` **struct** (24 bytes: ptr + len + cap) is now on the stack
- The actual string data `"hello"` is **still on the heap**
- We freed the memory where `Box0` stored the String struct, but not the string's data

**Before `into_inner()`:**

```bob
        STACK             |       HEAP
                          | 
   +---------------+      |     +------------------+
   | boxed: Box0   |      |     | String struct    |
   +---------------+      |     +------------------+
   | ptr: *--------+------+---->| "ptr:"*-----.    |
   +---------------+      |     | "len:"5      |   |
   8 bytes                |     | "cap:"5      |   |
                          |     +--------------+---+
                          |                    |
                          |                    v
                          |                  +---+---+---+---+---+
                          |                  | h | e | l | l | o |  "(5 bytes)"
                          |                  +---+---+---+---+---+
```

**After `into_inner()`:**

```bob
        STACK             |        HEAP
                          |  
   .---------------.      |      .------------------.
   | boxed: Box0   |      |      : String struct    :
   | "(consumed)"  |      |      : "(freed!)"       :
   '---------------'      |      '------------------'
                          |  
   +---------------+      |      
   | s: String     |      |      
   +---------------+      |      +---+---+---+---+---+
   | "ptr:"*-------+------+----->| h | e | l | l | o |  "(5 bytes)"
   | "len:"5       |      |      +---+---+---+---+---+
   | "cap:"5       |      |  
   +---------------+      |  
   24 bytes               |  
                          |  
```

The String still owns its heap-allocated data - we just moved the String struct itself from heap to stack!

