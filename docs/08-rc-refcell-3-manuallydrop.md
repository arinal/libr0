# ManuallyDrop & Anti-patterns

## Implementation: Why Rc Needs ManuallyDrop

### Quick Reminder: Rc Structure

From Chapter 7, recall that `Rc<T>` is defined as:

```rust
use std::cell::Cell;

struct RcInner<T> {
    strong_count: Cell<usize>,  // Reference count
    value: T,                   // The actual data
}

pub struct Rc<T> {
    ptr: *mut RcInner<T>,  // Pointer to heap allocation
}
```

When implementing `Drop` for `Rc<T>`, we face a **double-drop** problem. This affects `Rc<T>` for **any** type `T`, not just `RefCell`.

### The Problem: Double Drop

**Concrete example:** Let's say we have 2 strong references and 1 weak reference:

```rust
use std::rc::{Rc, Weak};

let strong1 = Rc::new(String::from("data"));
let strong2 = Rc::clone(&strong1);
let weak: Weak<String> = Rc::downgrade(&strong1);

// State: strong_count = 2, weak_count = 1
```

**What needs to happen when we drop both strong refs:**

1. Drop `strong1` → `strong_count` = 2 → 1 (value still alive)
2. Drop `strong2` → `strong_count` = 1 → 0 (value must be dropped now!)
3. But `weak` still exists! `RcInner` must stay allocated so `weak.upgrade()` can check `strong_count`
4. Later, drop `weak` → `weak_count` = 1 → 0 (now deallocate `RcInner`)

**The challenge:** In step 2, we must **drop the String** but **keep the RcInner allocation alive**. These are two separate operations.

### Naive Approach (Broken)

Now let's see the implementation problem:

```rust
impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        unsafe {
            // self.ptr is *mut RcInner<T>
            let inner = &*self.ptr;
            let count = inner.strong_count.get();
            inner.strong_count.set(count - 1);

            if count - 1 == 0 {
                // Last strong owner! (strong_count = 0, but weak_count = 1)

                // Step 1: Drop the value T
                ptr::drop_in_place(&mut (*self.ptr).value);
                // ✅ String is now dropped, its buffer freed

                // Step 2: Check if we can deallocate RcInner
                let weak = inner.weak_count.get();
                if weak == 0 {
                    // No Weak references, safe to deallocate
                    drop(Box::from_raw(self.ptr));
                    // ⚠️ PROBLEM: Box::drop will try to drop all fields of RcInner<T>
                    // This includes `value: T` - but we already dropped it in Step 1!
                    // 💥 DOUBLE DROP!
                } else {
                    // weak_count = 1, Weak reference still exists!
                    // We must keep the RcInner allocation alive so Weak::upgrade() can check strong_count
                    // The String is dropped, but RcInner stays allocated
                    // Later when Weak is dropped, it will deallocate RcInner
                }
            }
        }
    }
}
```

**Why we need TWO separate steps:**

When `strong_count` reaches 0:

1. **Drop the value** - No more strong owners, value must be freed
2. **Keep the RcInner allocation** - If `weak_count > 0`, Weak references still need to check `strong_count`

When `weak_count` ALSO reaches 0: 3. **Deallocate the RcInner** - No more references at all

**The double-drop problem:**

If we skip Step 1 and just do Step 3 (deallocate RcInner), we can't keep the allocation alive for Weak references. We need to drop the value when strong_count → 0, but deallocate the struct only when weak_count → 0.

Without `ManuallyDrop`, step 3 would automatically drop `value` again - but we already dropped it in step 1!

### The Solution: ManuallyDrop

`ManuallyDrop<T>` is a wrapper that tells Rust: "Don't automatically drop this value - I'll handle it manually."

**How does it do this?** Simple: by not implementing `Drop`. When Rust drops a struct, it automatically drops all its fields - but only if the field's type implements `Drop`. Since `ManuallyDrop<T>` doesn't implement `Drop`, Rust won't automatically drop the inner `T`.

```rust
use std::mem::ManuallyDrop;
use std::cell::Cell;

struct RcInner<T> {
    strong_count: Cell<usize>,
    weak_count: Cell<usize>,
    value: ManuallyDrop<T>,  // ← Wrapped in ManuallyDrop
}
```

Now with `ManuallyDrop`, the Drop implementation is safe:

```rust
impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = &*self.ptr;
            let count = inner.strong_count.get();
            inner.strong_count.set(count - 1);

            if count - 1 == 0 {
                // Manually drop the value (need mutable access)
                let inner_mut = &mut *self.ptr;
                ManuallyDrop::drop(&mut inner_mut.value);  // ✅ Drops T inside ManuallyDrop

                let weak = inner.weak_count.get();
                if weak == 0 {
                    // Deallocate the RcInner
                    // Since value is ManuallyDrop, Rust won't try to drop it again
                    drop(Box::from_raw(self.ptr));  // ✅ Safe! No double-drop
                }
            }
        }
    }
}
```

**Why it works:**

1. `ManuallyDrop<T>` prevents automatic drop when `RcInner` is deallocated
2. We explicitly call `ptr::drop_in_place()` on the value when `strong_count` reaches 0
3. No double drop - the value is dropped exactly once

### What is ManuallyDrop?

`ManuallyDrop<T>` is a zero-cost wrapper defined in `std::mem`:

```rust
#[repr(transparent)]
pub struct ManuallyDrop<T> {
    value: T,
}
```

**Key properties:**

- **Zero cost**: Same size and layout as `T` (due to `#[repr(transparent)]`)
- **Does not implement Drop**: Rust never automatically drops the inner `T`
- **Requires unsafe to drop**: You must explicitly call `ManuallyDrop::drop(&mut x)`

**Core operations:**

```rust
// Create
let x = ManuallyDrop::new(String::from("hello"));

// Access (no drop)
println!("{}", *x);  // Deref to access inner value

// Drop explicitly (unsafe - you must ensure no future access)
unsafe {
    ManuallyDrop::drop(&mut x);  // Drops the String
}
// After this, accessing x is undefined behavior!

// Move out without dropping (unsafe)
let s = unsafe { ManuallyDrop::take(&mut x) };  // Takes ownership of String
// Now s owns the String, x is left in uninitialized state
```

### Can We Manually Implement It?

Yes! Here's how `ManuallyDrop` is essentially implemented:

```rust
use std::mem::forget;
use std::ptr;

#[repr(transparent)]
pub struct ManualDrop<T> {
    value: T,
}

impl<T> ManualDrop<T> {
    pub const fn new(value: T) -> Self {
        ManualDrop { value }
    }

    // Safe: Just creates a reference
    pub fn deref(&self) -> &T {
        &self.value
    }

    // Unsafe: Caller must ensure no further access
    pub unsafe fn drop(slot: &mut Self) {
        // Read the value out and drop it
        ptr::drop_in_place(&mut slot.value);
    }

    // Unsafe: Caller must ensure no further access
    pub unsafe fn take(slot: &mut Self) -> T {
        // Move the value out, leaving uninitialized memory
        ptr::read(&slot.value)
    }
}

// No Drop implementation! This is the key.
// Without Drop, Rust won't automatically drop T.

impl<T> Deref for ManualDrop<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}
```

**Key insight:** By _not_ implementing `Drop`, the wrapper doesn't drop its contents. You must manually call `drop()` or `take()` to handle the inner value.

### ManuallyDrop in Practice

You'll see `ManuallyDrop` used whenever:

1. **Reference counting** - `Rc`, `Arc` need to control when `T` is dropped
2. **Custom allocators** - Managing memory lifetime manually
3. **FFI boundaries** - Passing ownership to C code
4. **Union types** - Only one variant should be dropped
5. **Implementing Drop flags** - Control drop order precisely

**Warning:** Using `ManuallyDrop` is subtle - you must ensure:

- Drop is called exactly once
- No access after drop
- Memory is eventually freed

Get it wrong, and you'll have memory leaks or use-after-free bugs!

## Anti-patterns

```rust
// ❌ BAD: Single owner, no sharing needed
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
data.borrow_mut().push(4);
// Should be: let mut data = vec![1, 2, 3];

// ❌ BAD: Never mutated, just shared
let config = Rc::new(RefCell::new(Config::load()));
// Should be: let config = Rc::new(Config::load());

// ❌ BAD: Using in multi-threaded context
let counter = Rc::new(RefCell::new(0));
thread::spawn(move || { ... });  // Won't compile! (Not Send)
// Should be: Arc<Mutex<i32>> or Arc<AtomicI32>
```

## Key Takeaways

1. **Rc<RefCell<T>> combines shared ownership + interior mutability** - Multiple owners can all mutate the same data
1. **Two levels of tracking** - Rc tracks reference counts (ownership), RefCell tracks borrows (usage)
1. **Two ways to fail** - Memory leaks from Rc cycles (safe but wasteful), panics from RefCell borrow violations (runtime error)
1. **Use Weak to break cycles** - Weak references don't keep values alive, preventing memory leaks in cyclic structures
1. **Parent-child pattern** - Use Rc for ownership direction (parent → child), Weak for back-references (child → parent)
1. **Drop borrows quickly** - Keep RefCell borrows short-lived to avoid runtime panics
1. **Don't overuse** - If you have a clear owner or don't need sharing, use simpler patterns (&mut T, Box<T>, Rc<T>)
1. **Always check Weak::upgrade()** - Weak references might point to dropped values, always handle None case
1. **Runtime cost** - Both Rc (reference counting) and RefCell (borrow checking) have runtime overhead, use only when needed

## Exercises

See [examples/08_rc_refcell.rs](../examples/08_rc_refcell.rs) for hands-on exercises demonstrating:

- Creating and using `Rc<RefCell<T>>`
- Multiple owners mutating shared data
- Creating and breaking reference cycles with `Weak`
- Building graphs and trees with cycles
- Observer pattern implementation
- Common pitfalls and how to avoid them

**Complete solutions:** Switch to the `answers` branch with `git checkout answers` to see completed exercises.

