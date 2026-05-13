# Building & Practice

## Building Our Own Cell

```rust
use std::cell::UnsafeCell;

pub struct Cell0<T> {
    value: UnsafeCell<T>,
}
```

### new - Create a Cell

```rust
impl<T> Cell0<T> {
    pub fn new(value: T) -> Cell0<T> {
        Cell0 {
            value: UnsafeCell::new(value),
        }
    }
}
```

### get - Copy the Value Out

```rust
impl<T: Copy> Cell0<T> {
    pub fn get(&self) -> T {
        // SAFETY: We only copy the value out, never give a reference
        unsafe { *self.value.get() }
    }
}
```

Note the `T: Copy` bound. This is crucial - we can only implement `get` for `Copy` types because we need to return a copy, not a reference.

### set - Replace the Value

```rust
impl<T> Cell0<T> {
    pub fn set(&self, value: T) {
        // SAFETY: We replace the entire value atomically (single-threaded)
        unsafe {
            *self.value.get() = value;
        }
    }
}
```

Notice `set` doesn't require `Copy` - we're replacing the value, not reading it.

### replace - Set and Return Old Value

```rust
impl<T> Cell0<T> {
    pub fn replace(&self, value: T) -> T {
        // SAFETY: We swap values without creating references
        unsafe {
            std::mem::replace(&mut *self.value.get(), value)
        }
    }
}
```

### take - Take the Value, Leave Default

```rust
impl<T: Default> Cell0<T> {
    pub fn take(&self) -> T {
        self.replace(T::default())
    }
}
```

### into_inner - Consume Cell, Get Value

```rust
impl<T> Cell0<T> {
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}
```

### get_mut - Common Confusion About Getting References from Cell

**Common confusion:** "Can Cell give me a reference to its inner value?"

**Short answer:** No, not through `&Cell`. That's the whole point of Cell - it can't give out references.

But Cell does have a `get_mut` method that returns `&mut T`. Here's the catch:

```rust
// Note: Separate impl block with ?Sized
impl<T: ?Sized> Cell0<T> {
    pub fn get_mut(&mut self) -> &mut T {  // Takes &mut self!
        self.value.get_mut()
    }
}
```

**The key insight:** `get_mut` requires `&mut self`, not `&self`.

This is just regular Rust borrowing - nothing special. The compiler catches it at compile time because `get_mut` takes `&mut self`.

This means you need **exclusive mutable access** to the Cell itself. At that point, you don't need interior mutability at all - Rust already knows at compile-time that you have exclusive access!

```rust
let mut cell = Cell0::new(5);  // Note: mut cell
*cell.get_mut() += 1;           // Direct mutable access
assert_eq!(cell.get(), 6);
```

**The compiler enforces normal borrow rules:**

```rust
let mut cell = Cell0::new(5);
let r1 = cell.get_mut();  // First mutable borrow
let r2 = cell.get_mut();  // ❌ Error: cannot borrow `cell` as mutable more than once

// Error message:
// cannot borrow `cell` as mutable more than once at a time
// first mutable borrow occurs here
```

**Why this defeats Cell's purpose:**

```bob
Cell's point:        &Cell<T>  --set()-->  mutate through &self
                                            (interior mutability)

get_mut:        &mut Cell<T>  --get_mut()-->  &mut T
                                               (normal mutability)
```

If you have `&mut Cell`, you could've just used `T` directly:

```rust
// Why use Cell if you have &mut anyway?
struct Config {
    value: Cell<i32>,  // Uses Cell...
}

impl Config {
    fn update(&mut self) {  // ...but needs &mut self?
        *self.value.get_mut() += 1;  // Could've just used i32!
    }
}

// More natural - no Cell needed:
struct Config {
    value: i32,  // Just use i32 directly
}

impl Config {
    fn update(&mut self) {
        self.value += 1;  // Simpler!
    }
}
```

**When would you actually use get_mut?**

Rarely. The only real use case is when you have a `Cell<T>` where `T` is not `Copy`, and you happen to have exclusive access to the Cell. At that point, `get_mut` lets you modify `T` in place without moving it out.

But this is uncommon - Cell exists precisely so you DON'T need `&mut`.

