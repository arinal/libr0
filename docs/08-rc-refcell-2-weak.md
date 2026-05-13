# Implementing Weak

## Building Weak\<T\>

> **Note:** The complete implementation is in [src/rc.rs](../../src/rc.rs) in this repository. That implementation already includes `Weak<T>` and uses `ManuallyDrop<T>` internally. These sections show how to build them step-by-step to help you understand how they work under the hood.

### Weak Structure

`Weak<T>` has the same structure as `Rc<T>` - it's just a pointer to the same `RcInner`:

```rust
use std::cell::Cell;

struct RcInner<T> {
    strong_count: Cell<usize>,
    weak_count: Cell<usize>,  // Track weak references
    value: T,
}

pub struct Rc<T> {
    ptr: *mut RcInner<T>,
}

pub struct Weak<T> {
    ptr: *mut RcInner<T>,  // Same pointer type as Rc!
}
```

**Key difference:** `Weak` points to the same allocation as `Rc`, but doesn't keep the value alive.

> **Implementation Note: Implicit Weak Reference**
> In the real `std::rc::Rc` implementation, `weak_count` starts at 1, not 0. This represents an "implicit weak reference" held by the `Rc` allocation itself. The count only reaches 0 when both:
>
> 1. All strong references are dropped (`strong_count == 0`)
> 2. All explicit weak references are dropped
>
> This pattern ensures the `RcInner` allocation stays valid as long as ANY reference (strong or weak) exists. When `strong_count` reaches 0, the value `T` is dropped but `RcInner` remains allocated with `weak_count = 1` (the implicit reference). Only when the last explicit `Weak` is dropped does `weak_count` go from 1 to 0, triggering deallocation of `RcInner`.
>
> For simplicity, our implementation here shows `weak_count` starting at 0, which is conceptually easier to understand but differs from the actual std library implementation.

### Creating Weak: Rc::downgrade()

Convert a strong reference to a weak one:

```rust
impl<T> Rc<T> {
    pub fn downgrade(this: &Rc<T>) -> Weak<T> {
        unsafe {
            let inner = &*this.ptr;

            // Increment weak_count
            let count = inner.weak_count.get();
            inner.weak_count.set(count + 1);

            // Create Weak pointing to the same allocation
            Weak { ptr: this.ptr }
        }
    }
}
```

**What happens:**

1. Increment `weak_count` (doesn't affect `strong_count`)
2. Create a new `Weak` with the same pointer
3. Value stays alive (strong_count unchanged)

### Upgrading Weak: Weak::upgrade()

Try to convert a weak reference back to a strong one:

```rust
impl<T> Weak<T> {
    pub fn upgrade(&self) -> Option<Rc<T>> {
        // Check for null pointer (from Weak::new())
        if self.ptr.is_null() {
            return None;
        }

        unsafe {
            let inner = &*self.ptr;
            let strong = inner.strong_count.get();

            // Check if value is still alive
            if strong == 0 {
                // Value has been dropped, can't upgrade
                None
            } else {
                // Value still alive, increment strong_count
                inner.strong_count.set(strong + 1);

                // Create new Rc pointing to same allocation
                Some(Rc { ptr: self.ptr })
            }
        }
    }
}
```

**Why it returns `Option`:**

- If `strong_count == 0`, all `Rc` owners have dropped, value is gone → return `None`
- If `strong_count > 0`, value still alive → increment count and return `Some(Rc)`

**Example:**

```rust
let strong = Rc::new(42);
let weak = Rc::downgrade(&strong);
// strong_count = 1, weak_count = 1

// Upgrade succeeds (strong_count > 0)
if let Some(rc) = weak.upgrade() {
    assert_eq!(*rc, 42);
    // strong_count now 2
}
// strong_count back to 1

drop(strong);
// strong_count = 0, value dropped

// Upgrade fails (strong_count == 0)
assert!(weak.upgrade().is_none());
```

### Weak::new() - Creating an Empty Weak

```rust
impl<T> Weak<T> {
    pub fn new() -> Weak<T> {
        Weak {
            ptr: std::ptr::null_mut(),  // Null pointer, doesn't point to anything
        }
    }
}
```

**Usage:** For fields that might never have a value:

```rust
struct Node {
    parent: Weak<Node>,  // Start with no parent
}

let root = Node {
    parent: Weak::new(),  // Root has no parent
};
```

### Weak's Drop Implementation

When a `Weak` is dropped, we decrement `weak_count` and potentially deallocate:

```rust
impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        // Check for null pointer (from Weak::new())
        if self.ptr.is_null() {
            return;
        }

        unsafe {
            let inner = &*self.ptr;
            let weak = inner.weak_count.get();
            inner.weak_count.set(weak - 1);

            if weak - 1 == 0 {
                // Last Weak reference!

                let strong = inner.strong_count.get();
                if strong == 0 {
                    // Value already dropped by Rc, safe to deallocate
                    drop(Box::from_raw(self.ptr));
                }
                // If strong > 0, some Rc still exists, they'll handle deallocation
            }
        }
    }
}
```

**The lifecycle:**

```rust
let strong1 = Rc::new(String::from("data"));
let strong2 = Rc::clone(&strong1);
let weak = Rc::downgrade(&strong1);
// strong_count = 2, weak_count = 1

drop(strong1);
// strong_count: 2 → 1
// Nothing deallocated

drop(strong2);
// strong_count: 1 → 0
// Rc::drop() drops the String
// But weak_count = 1, so RcInner stays allocated

drop(weak);
// weak_count: 1 → 0
// strong_count already 0, so Weak::drop() deallocates RcInner
```

**Key insight:** `Weak::drop()` only deallocates `RcInner` when BOTH counts are 0. It never drops the value - `Rc::drop()` handles that.

### Why Weak Doesn't Keep Value Alive

The key is in `upgrade()`:

```rust
pub fn upgrade(&self) -> Option<Rc<T>> {
    if strong_count == 0 {
        return None;  // Value gone, can't access it
    }
    // ...
}
```

**Contrast with `Rc::clone()`:**

```rust
impl<T> Clone for Rc<T> {
    fn clone(&self) -> Rc<T> {
        // Always increments strong_count
        let count = self.inner().strong_count.get();
        self.inner().strong_count.set(count + 1);
        Rc { ptr: self.ptr }
    }
}
```

**The difference:**

- `Rc::clone()` → increments `strong_count` → keeps value alive
- `Rc::downgrade()` → increments `weak_count` → doesn't keep value alive
- `Weak::upgrade()` → checks `strong_count` first → returns `None` if value dropped

### Complete Weak Example

```rust
use std::rc::{Rc, Weak};

// Create strong reference
let strong = Rc::new(42);
println!("strong_count: {}", Rc::strong_count(&strong)); // 1

// Create weak reference
let weak: Weak<i32> = Rc::downgrade(&strong);
println!("weak_count: {}", Rc::weak_count(&strong)); // 1

// Upgrade weak to strong (value still alive)
if let Some(upgraded) = weak.upgrade() {
    println!("Upgraded: {}", *upgraded); // 42
    println!("strong_count: {}", Rc::strong_count(&upgraded)); // 2
}
// upgraded dropped, strong_count back to 1

// Drop the strong reference
drop(strong);

// Try to upgrade (value is gone)
match weak.upgrade() {
    Some(_) => println!("Upgraded successfully"),
    None => println!("Can't upgrade, value dropped"), // This runs
}
```

