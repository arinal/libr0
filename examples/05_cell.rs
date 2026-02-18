//! Chapter 5: Cell - Exercises
//!
//! Complete the TODO items to practice using MyCell for interior mutability.
//! Run with: cargo run --example cell

#![allow(unused)]

#[macro_use]
mod common;

use rustlib::cell::MyCell;

// ============================================================================
// Exercises - Replace variables with TODOs with the correct MyCell operations
// ============================================================================

fn _01_new_and_get() {
    let cell = MyCell::new(42);
    let value = 0; // TODO: get the value from cell

    assert_eq!(value, 42);
}

fn _02_set() {
    let cell = MyCell::new(10);
    // TODO: set cell to 20

    assert_eq!(cell.get(), 20);
}

fn _03_shared_mutation() {
    let cell = MyCell::new(0);
    let ref1 = &cell;
    let ref2 = &cell;

    // TODO: use ref1 to set cell to 5
    // TODO: use ref2 to set cell to 10

    assert_eq!(cell.get(), 10);
    // Both shared references could mutate!
}

fn _04_replace() {
    let cell = MyCell::new(String::from("hello"));
    let old = String::new(); // TODO: replace cell contents with "world"

    assert_eq!(old, "hello");
    assert_eq!(cell.into_inner(), "world");
}

fn _05_swap() {
    let a = MyCell::new(1);
    let b = MyCell::new(2);

    // TODO: swap the values of a and b

    assert_eq!(a.get(), 2);
    assert_eq!(b.get(), 1);
}

fn _06_take() {
    let cell = MyCell::new(Some(42));
    let value: Option<i32> = None; // TODO: take the value from cell

    assert_eq!(value, Some(42));
    assert_eq!(cell.get(), None); // Default is None
}

fn _07_update() {
    let cell = MyCell::new(5);
    // TODO: update cell by doubling its value (use update method)

    assert_eq!(cell.get(), 10);
}

fn _08_into_inner() {
    let cell = MyCell::new(String::from("owned"));
    let value = String::new(); // TODO: consume cell and extract the value

    assert_eq!(value, "owned");
    // cell is no longer valid here
}

fn _09_clone() {
    let cell1 = MyCell::new(42);
    let cell2 = MyCell::new(0); // TODO: clone cell1

    cell1.set(100);

    assert_eq!(cell1.get(), 100);
    assert_eq!(cell2.get(), 42); // Independent copy
}

fn _10_as_ptr() {
    let cell = MyCell::new(99);
    let ptr: *mut i32 = std::ptr::null_mut(); // TODO: get raw pointer from cell

    let value = unsafe { *ptr };
    assert_eq!(value, 99);
}

fn _11_default() {
    let cell: MyCell<i32> = MyCell::new(0); // TODO: create cell using Default trait

    assert_eq!(cell.get(), 0);
}

// ============================================================================
// Real-world Demo: Counter with Interior Mutability
// ============================================================================

struct Counter {
    count: MyCell<usize>,
}

impl Counter {
    fn new() -> Counter {
        Counter {
            count: MyCell::new(0),
        }
    }

    // Note: Takes &self, not &mut self!
    fn increment(&self) {
        self.count.set(self.count.get() + 1);
    }

    fn get(&self) -> usize {
        self.count.get()
    }

    fn reset(&self) {
        self.count.set(0);
    }
}

fn _12_counter() {
    let counter: Counter = Counter::new(); // TODO: create new counter

    // Multiple shared references can all increment
    let r1 = &counter;
    let r2 = &counter;
    let r3 = &counter;

    // TODO: increment using r1, r2, r3

    let count = 0; // TODO: get final count

    assert_eq!(count, 3);
}

// ============================================================================
// Real-world Demo: Config with Interior Mutability
// ============================================================================

struct Config {
    max_retries: MyCell<u32>,
    timeout_ms: MyCell<u32>,
    debug_mode: MyCell<bool>,
}

impl Config {
    fn new() -> Config {
        Config {
            max_retries: MyCell::new(3),
            timeout_ms: MyCell::new(1000),
            debug_mode: MyCell::new(false),
        }
    }

    fn enable_debug(&self) {
        self.debug_mode.set(true);
    }

    fn set_retries(&self, retries: u32) {
        self.max_retries.set(retries);
    }
}

fn _13_config() {
    let config = Config::new();

    // Can modify through shared reference
    let cfg_ref = &config;
    // TODO: enable debug mode using cfg_ref
    // TODO: set max_retries to 5 using cfg_ref

    assert_eq!(config.debug_mode.get(), true);
    assert_eq!(config.max_retries.get(), 5);
}

// ============================================================================
// Advanced: Cache with Interior Mutability
// ============================================================================

struct Cache<T> {
    value: MyCell<Option<T>>,
}

impl<T: Copy> Cache<T> {
    fn new() -> Self {
        Cache {
            value: MyCell::new(None),
        }
    }

    fn get_or_compute<F: FnOnce() -> T>(&self, f: F) -> T {
        match self.value.get() {
            Some(v) => v,
            None => {
                let computed = f();
                self.value.set(Some(computed));
                computed
            }
        }
    }

    fn clear(&self) {
        self.value.set(None);
    }
}

fn _14_cache() {
    let cache: Cache<i32> = Cache::new(); // TODO: create new cache

    let mut call_count = 0;
    let expensive_fn = || {
        call_count += 1;
        42
    };

    let result1 = 0; // TODO: get value using get_or_compute
    let result2 = 0; // TODO: get value again (should be cached)

    assert_eq!(result1, 42);
    assert_eq!(result2, 42);
    assert_eq!(call_count, 1); // Only called once!

    // TODO: clear the cache

    let result3 = cache.get_or_compute(|| 99);
    assert_eq!(result3, 99);
}

fn _15_get_mut() {
    // get_mut gives you a real mutable reference
    // But you need &mut Cell for this, not just &Cell
    let mut cell = MyCell::new(5);

    // Direct mutable access - no copying needed
    *cell.get_mut() += 10;
    assert_eq!(cell.get(), 15);

    // Can pass the mutable reference to other functions
    fn increment(value: &mut i32) {
        *value += 1;
    }
    increment(cell.get_mut());
    assert_eq!(cell.get(), 16);

    // Works with non-Copy types too!
    let mut cell_vec = MyCell::new(vec![1, 2, 3]);
    cell_vec.get_mut().push(4);
    // Can't use get() here because Vec is not Copy
    // But we can use replace() or into_inner()
    assert_eq!(cell_vec.into_inner(), vec![1, 2, 3, 4]);

    // Why is get_mut rarely used?
    // If you have &mut Cell, you could've just used T directly!
    println!("get_mut is useful when you have &mut Cell, but that's rare");
    println!("Cell's main purpose is interior mutability through &Cell");
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    run_all![
        "MyCell",
        _01_new_and_get,
        _02_set,
        _03_shared_mutation,
        _04_replace,
        _05_swap,
        _06_take,
        _07_update,
        _08_into_inner,
        _09_clone,
        _10_as_ptr,
        _11_default,
        _12_counter,
        _13_config,
        _14_cache,
        _15_get_mut,
    ];
}