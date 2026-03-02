# Chapter 6: RefCell - Runtime Borrow Checking

## The Real Problem RefCell Solves

Remember `Cell<T>`? It works perfectly... for `Copy` types:

```rust
use std::cell::Cell;

let count = Cell::new(0);
count.set(count.get() + 1); // ✅ Works! i32 is Copy
```

But try this:

```rust
let name = Cell::new(String::from("hello"));
// name.get()  // ❌ ERROR: String doesn't implement Copy!
```

**The issue**: `Cell::get()` returns a **copy** of the value. This only works if `T: Copy`.

For non-Copy types like `String`, `Vec`, or your custom structs, we need a different approach. We need **references** to the data, not copies.

## What RefCell Actually Does

`RefCell<T>` gives you interior mutability for **any type** by doing one clever thing:

**It enforces Rust's borrowing rules at runtime instead of compile time.**

Instead of compilation errors, you get panics!

```rust
use std::cell::RefCell;

let cell = RefCell::new(String::from("hello"));

// Get a reference (not a copy!)
let borrowed: Ref<String> = cell.borrow();
println!("{}", *borrowed);  // ✅ Works!

// Mutate through a mutable reference
let mut borrowed_mut: RefMut<String> = cell.borrow_mut();
borrowed_mut.push_str(" world");  // ✅ Works!
```

The borrowing rules are **exactly the same**:

- Many `borrow()` calls can coexist (multiple readers)
- Only one `borrow_mut()` at a time (single writer)
- Can't have `borrow()` and `borrow_mut()` simultaneously (no reading while writing)

The **only difference**: violations cause a **runtime panic** instead of a compile error:

```rust
let cell = RefCell::new(5);

let r1 = cell.borrow();      // ✅ OK
let r2 = cell.borrow();      // ✅ OK - multiple immutable borrows
let r3 = cell.borrow_mut();  // 💥 PANIC! Can't borrow mutably while borrowed
```

### Don't Confuse borrow/borrow_mut with get_mut

**Important distinction:** `RefCell` has a `get_mut()` method, but it's **not the main point** of `RefCell` and is rarely used.

```rust
let mut cell = RefCell::new(5);  // Note: mut cell
*cell.get_mut() += 1;             // Takes &mut self - compile-time check!
```

**Key difference:**

- **`get_mut()`**: Requires `&mut self` → **compile-time checked** → Just regular Rust borrowing
- **`borrow()` / `borrow_mut()`**: Only need `&self` → **runtime checked** → This is interior mutability!

If you have `&mut RefCell<T>`, you don't need interior mutability at all - you already have exclusive access! You could've just used `T` directly. The whole point of `RefCell` is `borrow()` and `borrow_mut()` - they let you mutate through `&self`.

See the [Cell chapter's get_mut section](./05-cell.md:454) for more details on why `get_mut` defeats the purpose of interior mutability.

## Why Cell is Copy-only and RefCell is Not

This is crucial to understand:

**Cell** works by copying values in and out (`set`, `get`, `replace`):

- `set(value)` - Replaces the current value with a new copy
- `get()` - Returns a copy of the current value
- **Requires `Copy`** - since we're always duplicating values, not borrowing them

**RefCell** gives you references (`borrow`, `borrow_mut`):

- `borrow()` returns `&T` wrapped in a guard
- `borrow_mut()` returns `&mut T` wrapped in a guard
- **Works with any type** because we're just borrowing, not copying

```rust
// Cell: requires Copy
let cell = Cell::new(42);
let value = cell.get();  // Copies the value out

// RefCell: works with any type
let cell = RefCell::new(String::from("hello"));
let borrowed = cell.borrow();  // Returns &String (no copy!)
```

**TL;DR**: Use `Cell<T>` for small `Copy` types. Use `RefCell<T>` for everything else.

## How RefCell Tracks Borrows

RefCell uses a simple counter (`borrow_count`) to track the borrow state:

```
┌────────────────────────────────────┐
│ RefCell<T>                         │
│ ┌────────────────────────────────┐ │
│ │ borrow_count: Cell<isize>      │ │  ← The counter!
│ │                                │ │     0 = not borrowed
│ │ value: UnsafeCell<T>           │ │    >0 = # of immutable borrows
│ └────────────────────────────────┘ │    -1 = mutably borrowed
└────────────────────────────────────┘
```

How `borrow_count` changes:

- `borrow()` checks if `borrow_count >= 0`, then increments it
- `borrow_mut()` checks if `borrow_count == 0`, then sets it to -1
- Dropping the guard restores `borrow_count`

## Building Our Own RefCell

### The Structure

```rust
use std::cell::{Cell, UnsafeCell};

pub struct RefCell0<T> {
    borrow_count: Cell<isize>,  // 0, positive, or -1
    value: UnsafeCell<T>,
}

// Borrow states:
// 0      = not borrowed
// > 0    = immutably borrowed N times
// -1     = mutably borrowed
```

### The Guard Types

RefCell uses **two types of guards** to track borrows:

**`Ref<T>`** - Returned by `borrow()` for immutable access:
- Dereferences to `&T`
- Decrements the count when dropped

**`RefMut<T>`** - Returned by `borrow_mut()` for mutable access:
- Dereferences to `&mut T`
- Resets the count to 0 when dropped

```rust
pub struct Ref<'a, T> {
    refcell: &'a RefCell0<T>,
}

pub struct RefMut<'a, T> {
    refcell: &'a RefCell0<T>,
}
```

### new - Create RefCell

```rust
impl<T> RefCell0<T> {
    pub fn new(value: T) -> RefCell0<T> {
        RefCell0 {
            borrow_count: Cell::new(0),
            value: UnsafeCell::new(value),
        }
    }
}
```

### borrow - Immutable Access

```rust
impl<T> RefCell0<T> {
    pub fn borrow(&self) -> Ref<'_, T> {
        let count = self.borrow_count.get();
        if count < 0 {
            panic!("Already mutably borrowed!");
        }
        self.borrow_count.set(count + 1);
        Ref { refcell: self }
    }
}
```

### borrow_mut - Mutable Access

```rust
impl<T> RefCell0<T> {
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        let count = self.borrow_count.get();
        if count != 0 {
            panic!("Already borrowed!");
        }
        self.borrow_count.set(-1);
        RefMut { refcell: self }
    }
}
```

### Implementing the Guards

The guards need to do two things:
1. **Deref** - Allow transparent access to the inner value
2. **Drop** - Clean up the borrow count when done

**How deref is used in practice:**

```rust
let cell = RefCell0::new(String::from("hello"));

// borrow() returns Ref<String>, not &String
let borrowed: Ref<String> = cell.borrow();

// But we can use it like a &String thanks to Deref!
println!("{}", borrowed.len());        // Calls String::len
println!("{}", borrowed.to_uppercase()); // Calls String::to_uppercase
println!("{}", *borrowed);             // Explicit deref to &String
```

The `Ref<String>` guard **derefs** to `&String` automatically, so you can call all `String` methods on it!

**How Drop ensures cleanup:**

```rust
let cell = RefCell0::new(5);

{
    let borrowed = cell.borrow();  // borrow_count: 0 -> 1
    println!("{}", *borrowed);
} // borrowed dropped here - borrow_count: 1 -> 0 (Drop called!)

// Now we can borrow_mut because count is back to 0
let mut borrowed_mut = cell.borrow_mut(); // ✅ Works!
```

Without `Drop`, the count would stay at 1 forever, and you'd never be able to borrow mutably!

**Implementation:**

```rust
use std::ops::{Deref, DerefMut};

// Ref<T> derefs to &T
impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: borrow() already checked borrowing rules
        unsafe { &*self.refcell.value.get() }
    }
}

// When Ref<T> is dropped, decrement the count
impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        let count = self.refcell.borrow_count.get();
        self.refcell.borrow_count.set(count - 1);
    }
}

// RefMut<T> derefs to &T
impl<T> Deref for RefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.refcell.value.get() }
    }
}

// RefMut<T> also derefs to &mut T
impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: borrow_mut() already checked borrowing rules
        unsafe { &mut *self.refcell.value.get() }
    }
}

// When RefMut<T> is dropped, reset count to 0
impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        self.refcell.borrow_count.set(0);
    }
}
```

**Why the guards store `&RefCell0<T>`:**

The guards need a reference back to the `RefCell0` so they can update `borrow_count` when dropped. This is how RAII (Resource Acquisition Is Initialization) works - the cleanup happens automatically!

## try_borrow - Non-Panicking Version

Sometimes you want to try borrowing without panicking. `borrow()` and `borrow_mut()` panic on violations, but what if you want to handle the error gracefully?

**The problem with panicking:**

```rust
let cell = RefCell0::new(vec![1, 2, 3]);
let borrowed = cell.borrow();

// This panics! Program crashes.
let mut borrowed_mut = cell.borrow_mut(); // 💥 PANIC!
```

**Using try_borrow instead:**

```rust
let cell = RefCell0::new(vec![1, 2, 3]);
let borrowed = cell.borrow();

// Try to borrow mutably - returns Result instead of panicking
match cell.try_borrow_mut() {
    Ok(mut b) => {
        b.push(4);
        println!("Successfully modified!");
    }
    Err(_) => {
        println!("Can't borrow mutably right now, already borrowed!");
        // Handle the error gracefully - maybe retry later
    }
}
```

**Practical use case - Checking before borrowing:**

```rust
fn safely_update(cell: &RefCell0<Vec<i32>>, value: i32) {
    // Check if we can borrow mutably first
    if let Ok(mut data) = cell.try_borrow_mut() {
        data.push(value);
    } else {
        // Already borrowed, skip or queue the update
        println!("Skipping update, cell is busy");
    }
}
```

**Implementation:**

```rust
pub enum BorrowError {
    AlreadyMutablyBorrowed,
}

pub enum BorrowMutError {
    AlreadyBorrowed,
}

impl<T> RefCell0<T> {
    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        let count = self.borrow_count.get();
        if count < 0 {
            Err(BorrowError::AlreadyMutablyBorrowed)
        } else {
            self.borrow_count.set(count + 1);
            Ok(Ref { refcell: self })
        }
    }

    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        let count = self.borrow_count.get();
        if count != 0 {
            Err(BorrowMutError::AlreadyBorrowed)
        } else {
            self.borrow_count.set(-1);
            Ok(RefMut { refcell: self })
        }
    }
}
```

The logic is identical to `borrow()` and `borrow_mut()`, but returns `Result` instead of panicking!

## The Complete Implementation

See the full implementation in [refcell.rs](./src/refcell.rs).

## Common Patterns

### Pattern 1: Deferred Mutation

```rust
fn process_data(cell: &RefCell<Vec<i32>>) {
    // Read first
    let sum: i32 = cell.borrow().iter().sum();

    // Then mutate (borrow dropped, so this is safe)
    cell.borrow_mut().push(sum);
}
```

**Important**: Drop borrows as soon as possible to avoid panics!

### Pattern 2: Scoped Borrows

```rust
let cell = RefCell::new(vec![1, 2, 3]);

{
    let borrowed = cell.borrow();
    println!("Length: {}", borrowed.len());
} // borrowed dropped here

cell.borrow_mut().push(4); // Now safe to mutate
```

### Pattern 3: Early Return to Drop Borrow

```rust
fn check_and_modify(cell: &RefCell<Option<String>>) {
    // Check if we need to modify
    if cell.borrow().is_none() {
        return; // Borrow dropped on return
    }

    // Safe to mutate now
    *cell.borrow_mut() = Some(String::from("modified"));
}
```

## RefCell vs Cell vs Mutex

|                  | Cell           | RefCell       | Mutex     |
| ---------------- | -------------- | ------------- | --------- |
| Type restriction | `Copy` for get | Any           | Any       |
| Returns          | Copy           | Reference     | Guard     |
| Runtime cost     | None           | Counter check | Lock      |
| Panics?          | Never          | On violation  | On poison |
| Thread-safe?     | No             | No            | Yes       |

**Note:** `Mutex` will be covered in detail in a later chapter on thread-safe interior mutability.

## Pitfalls

### Pitfall 1: Holding Borrows Too Long

```rust
let cell = RefCell::new(vec![1, 2, 3]);
let borrowed = cell.borrow(); // Held across...
let len = borrowed.len();
cell.borrow_mut().push(4);    // PANIC! Still borrowed!
```

**Fix 1: Explicitly drop the borrow**

```rust
let cell = RefCell::new(vec![1, 2, 3]);
let borrowed = cell.borrow();
let len = borrowed.len();
drop(borrowed);               // ✅ Explicitly drop the borrow
cell.borrow_mut().push(4);    // Now safe!
```

**Fix 2: Use a shorter scope**

```rust
let cell = RefCell::new(vec![1, 2, 3]);

let len = {
    let borrowed = cell.borrow();
    borrowed.len()
};  // ✅ borrowed dropped here

cell.borrow_mut().push(4);    // Now safe!
```

**Fix 3: Don't store the borrow (best approach)**

```rust
let cell = RefCell::new(vec![1, 2, 3]);
let len = cell.borrow().len();  // ✅ Borrow dropped immediately
cell.borrow_mut().push(4);       // Now safe!
```

### Pitfall 2: Borrowing in a Loop

**The bad pattern:**

```rust
let cell = RefCell::new(0);
let mut borrows = Vec::new();

// BAD: Accumulating borrows without dropping them
for _ in 0..10 {
    let r = cell.borrow();
    borrows.push(r);  // Moves r into Vec - now the Vec owns it!
                      // r won't drop at the end of iteration
                      // It only drops when the Vec drops
}

// Later, trying to mutate:
cell.borrow_mut();  // 💥 PANIC! Still have 10 active borrows!
                    // All guards are still alive inside the Vec
```

Even this seemingly innocent code has a subtle issue:

```rust
let cell = RefCell::new(vec![1, 2, 3]);

let first_borrow = cell.borrow();  // This borrow lives outside the loop!
for i in 0..5 {
    let current = cell.borrow();
    println!("{:?}", *current);
    // current drops here - this is fine!
}
// But first_borrow is STILL active!
// If you try to borrow_mut here, it will panic:
cell.borrow_mut().push(4);  // 💥 PANIC! first_borrow is still active
```

**The fix:**

```rust
let cell = RefCell::new(vec![1, 2, 3]);

// ✅ Each borrow drops at the end of the iteration
for i in 0..5 {
    let borrowed = cell.borrow();
    println!("{:?}", *borrowed);
    // borrowed drops here automatically
}

// Now safe to borrow mutably
cell.borrow_mut().push(4);
```

Or even better - don't store the guard at all:

```rust
let cell = RefCell::new(vec![1, 2, 3]);

// ✅ Best: Borrow dropped immediately after use
for i in 0..5 {
    println!("{:?}", *cell.borrow());
}
```

### Pitfall 3: Recursive Calls with Borrows

```rust
fn recursive(cell: &RefCell<i32>, n: i32) {
    if n == 0 { return; }

    let _borrowed = cell.borrow(); // Borrow held...
    recursive(cell, n - 1);        // ...across recursive call
    // If recursive() tries to borrow_mut, PANIC!
}
```

## Key Takeaways

1. **RefCell doesn't bypass the borrow checker** - It moves it from compile time to runtime. Same rules, same restrictions.
2. **Cell = Copy types, RefCell = Any type** - Cell copies values, RefCell borrows references.
3. **Guards implement RAII** - The borrow is released when the guard drops. This is crucial!
4. **Keep borrows short-lived** - Drop guards as soon as possible to avoid panics.
5. **Use try_borrow for fallible access** - Better than panicking in complex scenarios.
6. **RefCell is NOT thread-safe** - Single-threaded only. Use `Mutex` or `RwLock` for concurrency.

## Exercises

Practice using RefCell with hands-on exercises in [06_refcell.rs](../examples/06_refcell.rs).

**Complete solutions:** Switch to the `answers` branch with `git checkout answers` to see completed exercises

## Next Chapter

[Rc](./07-rc.md) - Reference counting for shared ownership.
