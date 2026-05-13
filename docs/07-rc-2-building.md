# Building Our Own Rc

### The Inner Structure

Before diving into implementation details, we need to answer a fundamental question: **Why do we need `RcInner`? Why not just put the count in `Rc` itself?**

#### Why We Need RcInner: The Shared Count Problem

Consider what happens if we try to store the count directly in each `Rc`:

```rust
// ❌ WRONG: Each Rc has its own count
struct Rc0<T> {
    ptr: *mut T,
    count: usize,  // Each Rc instance has its own count!
}
```

**The problem:** When you clone an `Rc`, you create a **new struct** with a **separate count**:

```rust
let rc1 = Rc::new(String::from("data"));  // rc1.count = 1
let rc2 = rc1.clone();                    // rc2.count = 1 (copied!)

// rc1 and rc2 have DIFFERENT counts!
// When rc1 drops, it sees count = 1, so it frees the memory
// When rc2 drops, it also sees count = 1, so it tries to free again
// 💀 Double free! Undefined behavior!
```

**What we actually need:** All `Rc` instances pointing to the same data must share the **SAME count**:

```rust
let rc1 = Rc::new(String::from("data"));
let rc2 = rc1.clone();
let rc3 = rc1.clone();

// All three need to see: count = 3
// When rc1 drops: count becomes 2
// When rc2 drops: count becomes 1
// When rc3 drops: count becomes 0 → NOW free the memory
```

**The solution: Store the count with the data, not with the pointer**

```rust
// ✅ CORRECT: Separate inner struct
struct RcInner<T> {
    strong_count: usize,  // Shared count, stored with the value
    value: T,
}

struct Rc0<T> {
    ptr: *mut RcInner<T>,  // Just a pointer to the shared data + count
}
```

**Visual comparison:**

**Wrong approach (count in each Rc):**

```bob
Stack                                    Heap
+-------------+                         +---------+
| rc1         |                         |  data   |
| ptr *-------+------------------------>|         |
| "count:"1   |   Each Rc has           +---------+
+-------------+   its own count!             ^
                  They can"'t"               |
+-------------+   coordinate!                |
| rc2         |                              |
| ptr *-------+------------------------------+
| "count:"1   |   ❌  WRONG
+-------------+
```

**Correct approach (count with data):**

```bob
Stack                                    Heap
+-------------+                         +--------------+
| rc1         |                         | "count:"2    | <- Shared count!
| ptr *-------+------------------------>| data         |
+-------------+                         +--------------+
                                               ^
+-------------+                                |
| rc2         |                                |
| ptr *-------+--------------------------------+
+-------------+   Both point to the SAME count
                  ✅  CORRECT
```

**In code:**

```rust
// Each Rc is just a pointer
let rc1 = Rc::new(String::from("data"));
// Heap: RcInner { count: 1, value: "data" }

let rc2 = rc1.clone();
// rc2 gets a COPY of the pointer (not the count!)
// Both rc1.ptr and rc2.ptr point to the SAME RcInner
// RcInner.count is incremented: count = 2

drop(rc1);
// Follows rc1.ptr to the shared RcInner
// Decrements the shared count: count = 1
// Doesn't free (count != 0)

drop(rc2);
// Follows rc2.ptr to the SAME RcInner
// Decrements the shared count: count = 0
// NOW frees the memory ✅
```

**Summary:** `RcInner` exists to ensure the count lives **with the data** on the heap, not **with the pointer** on the stack. This way, all `Rc` instances pointing to the same data share the exact same count.

---

Now let's tackle the main challenge in implementing `RcInner`.

#### The Challenge: Mutating Counts Through Shared References

**The problem:** When you clone an `Rc`, you only have `&self` (shared reference), but you need to increment the count.

If we tried using a plain `usize`:

```rust
struct RcInner<T> {
    strong_count: usize,  // ❌ Can't mutate this from &self
    value: T,
}

impl<T> Clone for Rc0<T> {
    fn clone(&self) -> Rc0<T> {
        let inner = unsafe { &*self.ptr };
        inner.strong_count += 1;  // ❌ ERROR: cannot mutate through shared reference!
        Rc0 { ptr: self.ptr }
    }
}
```

This fails because:

- `clone()` receives `&self` (the `Clone` trait requires this)
- From `&self`, we get `&RcInner<T>` (shared reference to inner data)
- Rust forbids mutation through shared references (prevents data races)
- But we MUST increment the count!

**The solution: `Cell<usize>`**

`Cell` provides **interior mutability** for `Copy` types (covered in Chapter 5):

```rust
struct RcInner<T> {
    strong_count: Cell<usize>,  // ✅ Can mutate through &self!
    value: T,
}

impl<T> Clone for Rc0<T> {
    fn clone(&self) -> Rc0<T> {
        let inner = unsafe { &*self.ptr };
        // ✅ Works! Cell allows mutation through shared reference
        inner.strong_count.set(inner.strong_count.get() + 1);
        Rc0 { ptr: self.ptr }
    }
}
```

#### The Simplified Structure

For now, we'll keep our implementation simple:

```rust
use std::cell::Cell;

struct RcInner<T> {
    strong_count: Cell<usize>,  // Cell: mutate through &self
    value: T,                    // The actual data
}

struct Rc0<T> {
    ptr: *mut RcInner<T>,
}
```

**Note:** The actual implementation in [src/rc.rs](../src/rc.rs) is more complete - it includes `weak_count` and uses `ManuallyDrop<T>` to support `Weak` references (non-owning pointers that don't keep the value alive). We'll cover `Weak` in Chapter 13 (Rc + RefCell).

### new - Create with Count 1

```rust
impl<T> Rc0<T> {
    fn new(value: T) -> Rc0<T> {
        let inner = Box::new(RcInner {
            strong_count: Cell::new(1),
            value,
        });

        Rc0 {
            ptr: Box::into_raw(inner),
        }
    }
}
```

### clone - Increment Count

```rust
impl<T> Clone for Rc0<T> {
    fn clone(&self) -> Rc0<T> {
        let inner = unsafe { &*self.ptr };
        inner.strong_count.set(inner.strong_count.get() + 1);
        Rc0 { ptr: self.ptr }
    }
}
```

**Important**: `Rc::clone()` is cheap! It just increments a counter. This is different from `.clone()` on the inner type which might be expensive.

Convention: Use `Rc::clone(&rc)` instead of `rc.clone()` to make it clear you're cloning the pointer, not the data.

### Deref - Access the Value

```rust
impl<T> Deref for Rc0<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &(*self.ptr).value }
    }
}
```

Note: `Rc` only gives you `&T` (shared reference), never `&mut T`. This is intentional - if multiple owners could mutate, we'd have data races.

### drop - Decrement and Maybe Free

```rust
impl<T> Drop for Rc0<T> {
    fn drop(&mut self) {
        let inner = unsafe { &*self.ptr };
        let count = inner.strong_count.get();
        inner.strong_count.set(count - 1);

        if count == 1 {
            // Last reference - deallocate everything
            unsafe {
                drop(Box::from_raw(self.ptr));
            }
        }
    }
}
```

When the last `Rc` drops (count becomes 0), we deallocate the entire `RcInner`, which automatically drops the value.

### strong_count - Check Reference Count

```rust
impl<T> Rc0<T> {
    fn strong_count(this: &Rc0<T>) -> usize {
        unsafe { (*this.ptr).strong_count.get() }
    }
}
```

### get_mut - Unique Access for Sole Owner

If you're the only owner (`strong_count == 1`), you can get `&mut T`:

```rust
impl<T> Rc0<T> {
    fn get_mut(this: &mut Rc0<T>) -> Option<&mut T> {
        if Rc0::strong_count(this) == 1 {
            // SAFETY: We're the sole owner, so no aliases exist
            unsafe { Some(&mut (*this.ptr).value) }
        } else {
            None
        }
    }
}
```

## The Complete Implementation

**Note:** The actual implementation in [src/rc.rs](../src/rc.rs) is more complete - it includes full `Weak0<T>` support and uses `ManuallyDrop<T>` to properly separate dropping the value from deallocating the memory. We'll explore `Weak` references in depth in Chapter 13 (Rc + RefCell). For this chapter, here's the complete `Rc` implementation:

```rust
use std::cell::Cell;
use std::ops::Deref;

struct RcInner<T> {
    strong_count: Cell<usize>,
    value: T,
}

pub struct Rc0<T> {
    ptr: *mut RcInner<T>,
}

impl<T> Rc0<T> {
    pub fn new(value: T) -> Rc0<T> {
        let inner = Box::new(RcInner {
            strong_count: Cell::new(1),
            value,
        });
        Rc0 { ptr: Box::into_raw(inner) }
    }

    pub fn strong_count(this: &Rc0<T>) -> usize {
        unsafe { (*this.ptr).strong_count.get() }
    }

    pub fn get_mut(this: &mut Rc0<T>) -> Option<&mut T> {
        if Rc0::strong_count(this) == 1 {
            unsafe { Some(&mut (*this.ptr).value) }
        } else {
            None
        }
    }
}

impl<T> Clone for Rc0<T> {
    fn clone(&self) -> Rc0<T> {
        let inner = unsafe { &*self.ptr };
        inner.strong_count.set(inner.strong_count.get() + 1);
        Rc0 { ptr: self.ptr }
    }
}

impl<T> Deref for Rc0<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &(*self.ptr).value }
    }
}

impl<T> Drop for Rc0<T> {
    fn drop(&mut self) {
        let inner = unsafe { &*self.ptr };
        let count = inner.strong_count.get();
        inner.strong_count.set(count - 1);

        if count == 1 {
            // Last reference - deallocate everything
            unsafe {
                drop(Box::from_raw(self.ptr));
            }
        }
    }
}
```

