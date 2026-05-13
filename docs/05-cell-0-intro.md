# Chapter 5: Cell - Interior Mutability

You want to track how many times a value gets accessed. Simple, right? Add a counter, increment it on every read. But Rust says no:

```rust
struct Stats {
    value: i32,
    access_counter: usize,
}

impl Stats {
    fn get_value(&self) -> i32 {
        self.access_counter += 1;  // ❌ can't mutate through &self
        self.value
    }
}
```

The problem? `get_value` takes `&self` (shared reference), but you need `&mut self` to change `access_counter`.

**"Just use `&mut self` then!"**

Can't. That would be terrible:

```rust
fn get_value(&mut self) -> i32 {  // Now requires &mut self
    self.access_counter += 1;
    self.value
}

// This breaks:
let stats = Stats { value: 42, access_counter: 0 };
let r1 = &stats;
let r2 = &stats;
let v1 = r1.get_value();  // ❌ can't call get_value on &Stats
let v2 = r2.get_value();  // Need &mut, can't have multiple readers!
```

You just killed multiple readers. The whole point of `&self` is that many readers can coexist. But now you can't read without exclusive access, all because of a silly counter.

The counter is just **bookkeeping**. It's not the meaningful data. The meaningful data (`value`) never changes. You want the struct to be **logically immutable** (value doesn't change) but **physically mutable** (counter does change).

## The Unsafe Escape Hatch

Fine. Let's use raw pointers:

```rust
fn get_value(&self) -> i32 {
    unsafe {
        let counter_ptr = &self.access_counter as *const usize as *mut usize;
        *counter_ptr += 1;
    }
    self.value
}
```

This works. You mutate through a shared reference by casting away the constness. But this is a common pattern. You'll need it for counters, caches, lazy initialization—anywhere you want interior mutability. Every time you write this unsafe code again, you're making a promise to Rust: "Trust me, this is safe." You verify it once in Stats. Then again in Cache. Then in LazyCell. Then in RefCount. Exhausting.

## Enter Cell: The Safe Wrapper

What if someone already wrote that unsafe code, verified it's sound, and wrapped it in a safe API? Then you'd never think about it again:

```rust
use std::cell::Cell;

struct Stats {
    value: i32,
    access_counter: Cell<usize>,  // ✅ Wrapped in Cell!
}

impl Stats {
    fn get_value(&self) -> i32 {
        // Mutate through &self - totally safe!
        self.access_counter.set(self.access_counter.get() + 1);
        self.value
    }
}
```

Done. No unsafe. No mental overhead. Just works.

**That's `Cell`**: a safe wrapper for interior mutability. It lets you mutate data through a shared reference without violating Rust's safety guarantees.
There are also other types for interior mutability for different use cases: `RefCell`, `Mutex`, etc that we'll cover later.

## How Cell Stays Safe

Let's try building a naive `ClumsyCell` that gives you references:

```rust
use std::cell::UnsafeCell;

struct ClumsyCell<T> {
    value: UnsafeCell<T>,
}

impl<T> ClumsyCell<T> {
    fn new(value: T) -> Self {
        ClumsyCell { value: UnsafeCell::new(value) }
    }

    fn get_ref(&self) -> &T {
        unsafe { &*self.value.get() }
    }

    fn set(&self, value: T) {
        unsafe { *self.value.get() = value; }
    }
}

// This compiles! But it's unsound.
let cell = ClumsyCell::new(5);
let r1: &i32 = cell.get_ref();     // Get reference to inner value

println!("{}", r1);  // Reads 5
cell.set(10);        // Mutate through &cell
println!("{}", r1);  // DANGER: r1 still exists but value changed!
                     // r1 is supposed to be immutable, but the data it points to changed
```

The problem: `r1` is an immutable reference, but the value it points to changed. That's exactly what Rust's borrow checker prevents. `ClumsyCell` bypassed it with `unsafe`, breaking Rust's safety guarantees.

**Real Cell's solution: never give you a reference to the inner value.**

```rust
let cell = Cell::new(5);
cell.set(10);           // ✅ Replace the value
let value = cell.get(); // ✅ Get a COPY of the value (requires T: Copy)
```

No references = no aliasing violations. You can't have a reference to something that might change, because you never get a reference at all. Only copies.

This is why **Cell only works with `Copy` types** (for `get()`). Can't copy out a `String` or `Vec`. For those, you need `RefCell` (next chapter).

**What Cell gives you:**

- **get()**: Copy the value out (requires `T: Copy`)
- **set()**: Replace the value entirely
- **replace()**: Swap values, return the old one
- **take()**: Take the value, leave `Default::default()` behind

All through `&Cell<T>`, not `&mut Cell<T>`. That's the magic.

## Motivation: When You Need to Mutate Through Shared References

When the struct is **mostly read-only** (`value` never changes), but a **minor field** (`access_count`) needs to change. Making the entire struct mutable just for this tracking would be too awkward and restrictive.

This is **logical vs physical constness**:

- **Logically const**: The meaningful data (`value`) doesn't change
- **Physically mutable**: Internal bookkeeping (`access_count`) does change

**Solution:** Use interior mutability for the counter while keeping `&self`. This lets you mutate the minor parts (bookkeeping, caches, counters) without requiring `&mut self` for the whole struct.

All interior mutability types are wrappers around `UnsafeCell` (which we'll explore next). Rust provides safe wrappers that handle the unsafe operations for you. We'll implement these wrappers (`Cell`, `RefCell`) later in this chapter to understand how they work.

| Type         | Check                 | Best for                           |
| ------------ | --------------------- | ---------------------------------- |
| `Cell<T>`    | None (Copy semantics) | Simple `Copy` types                |
| `RefCell<T>` | Runtime               | Any type, single-threaded          |
| `Mutex<T>`   | Runtime + blocking    | Any type, multi-threaded           |
| `RwLock<T>`  | Runtime + blocking    | Read-heavy, multi-threaded         |
| `Atomic*`    | Hardware              | Primitive integers, multi-threaded |

This chapter covers `Cell`. We'll cover `RefCell` in the next chapter.

## UnsafeCell: The Foundation

Every interior mutability type is built on `UnsafeCell<T>`. It's the **only** way to get mutability through a shared reference in Rust.

```rust
use std::cell::UnsafeCell;

let cell = UnsafeCell::new(5);
let ptr: *mut i32 = cell.get();  // Returns raw mutable pointer

// UNSAFE: We must ensure no aliasing
unsafe {
    *ptr = 10;
}
```

`UnsafeCell` is unsafe because:

- It gives you a `*mut T` from an `&UnsafeCell<T>`
- **You** are responsible for ensuring no data races or aliasing violations
- The compiler can't help you

That's why we have safe wrappers like `Cell` and `RefCell`.

### Can We Implement Our Own UnsafeCell?

**No.** `UnsafeCell` is a **compiler intrinsic** - it's deeply integrated into Rust's type system.

Here's what it conceptually looks like:

```rust
#[repr(transparent)]
pub struct UnsafeCell<T> {
    value: T,
}

impl<T> UnsafeCell<T> {
    pub fn new(value: T) -> UnsafeCell<T> {
        UnsafeCell { value }
    }

    pub fn get(&self) -> *mut T {
        &self.value as *const T as *mut T
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}
```

**But this naive implementation is wrong!** The compiler treats `UnsafeCell` specially:

1. **Disables certain optimizations** - The compiler won't assume immutability through `&UnsafeCell<T>`
2. **Allows interior mutability** - Without `UnsafeCell`, getting `*mut T` from `&T` is undefined behavior
3. **Memory model implications** - Tells LLVM that mutations can happen through shared references

If you tried this with a regular struct, the compiler might:

- Optimize away your writes (assumes `&T` means immutable)
- Reorder operations incorrectly
- Generate incorrect code

**Example of why it matters:**

```rust
// Regular struct - WRONG!
struct NotUnsafeCell<T> {
    value: T,
}

impl<T> NotUnsafeCell<T> {
    fn get(&self) -> *mut T {
        &self.value as *const T as *mut T
    }
}

let x = NotUnsafeCell { value: 5 };
let ptr = x.get();

// Compiler sees &x (shared ref) and might assume x.value never changes
let a = x.value;  // Reads 5
unsafe { *ptr = 10; }  // Mutate through pointer - UB!
let b = x.value;  // Compiler might optimize this to still be 5!
```

With the real `UnsafeCell`, the compiler knows mutation can happen and won't make those assumptions.

**Does Rust have a `volatile` keyword?**

No. In C/C++, `volatile` tells the compiler "don't optimize reads/writes to this variable." Rust doesn't have a `volatile` keyword, but instead provides:

- **`UnsafeCell`**: For interior mutability (single-threaded mutation through shared references)
- **`std::ptr::read_volatile` / `write_volatile`**: Unsafe functions for cases where you need volatile semantics (e.g., memory-mapped I/O, hardware registers)
- **Atomics**: For thread-safe mutation with proper synchronization

`UnsafeCell` is not the same as `volatile` - it just tells the compiler "this can be mutated through shared references," while volatile prevents _all_ optimizations. Most Rust code uses `UnsafeCell` (via `Cell`, `RefCell`, etc.), not volatile operations.

**Bottom line:** You **must** use `std::cell::UnsafeCell`. It's the only sound way to implement interior mutability.

## What is Cell?

`Cell<T>` is a safe wrapper around `UnsafeCell<T>` for **Copy types**:

```rust
use std::cell::Cell;

let cell = Cell::new(5);
cell.set(10);           // Mutate through shared reference!
let value = cell.get(); // Get a COPY of the value
```

Key insight: `Cell` never gives you a reference to the inner value. It only lets you:

- **get**: Copy the value out
- **set**: Replace the value entirely

This is safe because you can't have a reference to something that might change - you only ever have copies.

**What happens when references escape?**

If Cell gave you a reference, you'd have:

1. Multiple `&Cell` (shared references to the Cell itself) ✓ Allowed
2. A `&T` (reference to the inner value) ✓ Should be valid
3. But Cell can mutate through `&self`! ✗ Breaks Rust's aliasing rules!

```rust
// Hypothetical broken Cell with get_ref:
let cell = Cell::new(5);
let r1: &i32 = cell.get_ref();     // Get reference to inner value

println!("{}", r1);  // Reads 5
cell.set(10);        // Mutate through &self
println!("{}", r1);  // DANGER: r1 still pointing to the value inside the cell!
                     // In Rust with UB, optimizer might assume still 5!
                     // While it's actually 10
```

**Visualizing the problem:**

**Step 1: `cell.get_ref()` returns a reference**

```bob
+---------------+
| Cell<i32>     |
| +-----------+ |
| | value: 5  | | <----- r1: &i32 points here
| +-----------+ |        Remember, &i32 is immutable!
+---------------+
```

**Step 2: `cell.set(10)` changes the value**

```bob
+---------------+
| Cell<i32>     |
| +-----------+ |
| | value: 10 | | <----- r1: &i32 STILL points here!
| +-----------+ |        But r1 is supposed to be immutable!
+---------------+        The data it points to changed!
```

**BROKEN:** r1 points into Cell's memory, and that memory can be mutated.
This violates Rust's aliasing rules: you have an immutable reference (&i32)
to data that's being mutated.

The problem: `r1` is supposed to be immutable, but the value it points to changed!

**How Cell avoids this:**

```rust
// The real Cell
let cell = Cell::new(5);
let n: i32 = cell.get();  // n is a COPY of the value, not a reference

println!("{}", n);  // Reads 5
cell.set(10);       // Mutate through &self
println!("{}", n);  // Still reads 5, because n is a copy, not a reference!
```

**Initial state:**

```bob
+--------------+
| Cell<i32>    |
| +----------+ |
| | value: 5 | |  <- Value lives inside Cell
| +----------+ |
+--------------+
```

**`cell.get()` - Returns a COPY:**

```bob
+--------------+
| Cell<i32>    |
| +----------+ |    copy    +----+
| | value: 5 | | ---------> | 5  |  <- Copy created in new memory location
| +----------+ |            +----+    no reference to Cell's internal value!
+--------------+
```

**`cell.set(10)` - REPLACES the value:**

```bob
+---------------+
| Cell<i32>     |
| +-----------+ |            +----+
| | value: 10 | |            | 5  |  <- Old value still 5
| +-----------+ |            +----+
+---------------+
```

**Key insight**: You never get `&i32` or `&mut i32` pointing to the value inside Cell's box. Only copies come out. The inner value never escapes as a reference.

