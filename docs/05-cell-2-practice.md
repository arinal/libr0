# Practice & Thread Safety

## Cell in Practice: Simple Examples

**Example 1: Counter**

```rust
use std::cell::Cell;

struct HitCounter {
    count: Cell<usize>,
}

impl HitCounter {
    fn new() -> Self {
        HitCounter { count: Cell::new(0) }
    }

    fn record_hit(&self) {
        self.count.set(self.count.get() + 1);
    }

    fn get_count(&self) -> usize {
        self.count.get()
    }
}

// Usage
let counter = HitCounter::new();
counter.record_hit();  // count: 0 -> 1
counter.record_hit();  // count: 1 -> 2
counter.record_hit();  // count: 2 -> 3
counter.get_count()    // 3
```

**Example 2: Toggle Flag**

```rust
struct Toggle {
    state: Cell<bool>,
}

impl Toggle {
    fn new() -> Self {
        Toggle { state: Cell::new(false) }
    }

    fn toggle(&self) {
        self.state.set(!self.state.get());
    }

    fn is_on(&self) -> bool {
        self.state.get()
    }
}

// Usage
let toggle = Toggle::new();
toggle.is_on()   // false
toggle.toggle();
toggle.is_on()   // true
toggle.toggle();
toggle.is_on()   // false
```

**Example 3: Lazy Initialization**

```rust
struct LazyValue {
    initialized: Cell<bool>,
    value: Cell<i32>,
}

impl LazyValue {
    fn new() -> Self {
        LazyValue {
            initialized: Cell::new(false),
            value: Cell::new(0),
        }
    }

    fn get_or_init(&self, compute: impl FnOnce() -> i32) -> i32 {
        if !self.initialized.get() {
            let val = compute();
            self.value.set(val);
            self.initialized.set(true);
        }
        self.value.get()
    }
}

// Usage
let lazy = LazyValue::new();
let result1 = lazy.get_or_init(|| 42);  // Computes: 42
let result2 = lazy.get_or_init(|| 99);  // Returns cached: 42
```

All these examples mutate state through `&self` (shared reference) - impossible without `Cell`!

## Cell and Thread Safety: Send and Sync

`Cell<T>` is **not thread-safe**. It can be used in a single thread, but cannot be safely shared between threads.

**Quick overview:**

Rust has two special marker traits for thread safety:

- **`Send`**: A type can be transferred between threads (moved to another thread)
- **`Sync`**: A type can be shared between threads (multiple threads can have `&T`)

```rust
// Cell<T> is Send (can be moved between threads)
let cell = Cell::new(42);
std::thread::spawn(move || {
    cell.set(100);  // ✅ OK - moved to this thread
});

// Cell<T> is NOT Sync (cannot be shared between threads)
let cell = Cell::new(42);
std::thread::spawn(|| {
    cell.set(100);  // ❌ Cell is not Sync
});
```

**Why is Cell not Sync?**

If two threads could share `&Cell<T>`, they could both call `set()` simultaneously:

1. Thread 1: `cell.set(10)`
2. Thread 2: `cell.set(20)`
3. **Data race!** Both write to the same memory without synchronization

**Hypothetical example if Cell was Sync (this won't compile!):**

```rust
use std::cell::Cell;
use std::thread;

// Imagine Cell<T> was Sync (it's not!)
let counter = Cell::new(0);

// Try to share it between threads (won't compile)
let handle1 = thread::spawn(|| {
    for _ in 0..1000 {
        counter.set(counter.get() + 1);  // Thread 1 increments
    }
});

let handle2 = thread::spawn(|| {
    for _ in 0..1000 {
        counter.set(counter.get() + 1);  // Thread 2 increments
    }
});

handle1.join().unwrap();
handle2.join().unwrap();

// Expected: 2000
// Actual: Could be anything! (if data races were allowed)
// Both threads read, modify, write with no synchronization
println!("{}", counter.get());  // Undefined behavior!
```

`Cell` provides no internal synchronization, so it's unsafe for concurrent access. For thread-safe interior mutability, use:

- **`Mutex<T>`** or **`RwLock<T>`** - Provides locking
- **Atomics** (`AtomicUsize`, `AtomicBool`, etc.) - Hardware-level synchronization

**Note:** `Send` and `Sync` are covered in depth in a later chapter on concurrency. For now, just remember: `Cell` = single-threaded only.

## The Complete Implementation

See the full implementation in [cell.rs](./src/cell.rs).

## Cell vs Other Interior Mutability Types

|              | Cell                     | RefCell                  |
| ------------ | ------------------------ | ------------------------ |
| Works with   | `Copy` types (for `get`) | Any type                 |
| Returns      | Copy of value            | Reference (`Ref<T>`)     |
| Overhead     | None                     | Runtime borrow tracking  |
| Panic?       | Never                    | Yes, on borrow violation |
| Thread-safe? | No                       | No                       |

Use `Cell` when:

- Your type is `Copy` (integers, bools, small structs)
- You just need to get/set the value
- You want zero runtime overhead

Use `RefCell` when:

- Your type isn't `Copy`
- You need references to the inner value
- You're willing to pay for runtime borrow checking

## Key Takeaways

1. **Interior mutability** allows mutation through shared references
2. **UnsafeCell** is the primitive - unsafe but flexible
3. **Cell** is safe by never exposing references - only copies
4. **Use Cell for counters, flags, and simple state** - like `Rc`'s reference count
5. **Cell is not thread-safe** - use atomics or mutexes for that

## Exercises

See [exercises](./examples/05_cell.rs).

**Complete solutions:** Switch to the `answers` branch with `git checkout answers` to see completed exercises

