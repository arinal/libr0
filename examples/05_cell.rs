//! Chapter 4: Cell - Interior Mutability
//!
//! Run with: cargo run --example cell

use std::cell::UnsafeCell;

pub struct MyCell<T> {
    value: UnsafeCell<T>,
}

// Cell is !Sync - can't be shared between threads
// This is automatically inferred from UnsafeCell

impl<T> MyCell<T> {
    pub fn new(value: T) -> MyCell<T> {
        MyCell {
            value: UnsafeCell::new(value),
        }
    }

    pub fn set(&self, value: T) {
        // SAFETY: Single-threaded, no references escape
        unsafe {
            *self.value.get() = value;
        }
    }

    pub fn replace(&self, value: T) -> T {
        // SAFETY: Single-threaded, no references escape
        unsafe { std::mem::replace(&mut *self.value.get(), value) }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    pub fn as_ptr(&self) -> *mut T {
        self.value.get()
    }

    // Exercise: swap two cells
    pub fn swap(&self, other: &MyCell<T>) {
        // SAFETY: Single-threaded, no references escape
        unsafe {
            std::ptr::swap(self.value.get(), other.value.get());
        }
    }
}

impl<T: Copy> MyCell<T> {
    pub fn get(&self) -> T {
        // SAFETY: We only copy out, never expose a reference
        unsafe { *self.value.get() }
    }

    // Exercise: update
    pub fn update<F: FnOnce(T) -> T>(&self, f: F) {
        let old = self.get();
        self.set(f(old));
    }
}

impl<T: Default> MyCell<T> {
    pub fn take(&self) -> T {
        self.replace(T::default())
    }
}

impl<T: Clone> Clone for MyCell<T> {
    fn clone(&self) -> MyCell<T> {
        unsafe { MyCell::new((*self.value.get()).clone()) }
    }
}

impl<T: Default> Default for MyCell<T> {
    fn default() -> MyCell<T> {
        MyCell::new(T::default())
    }
}

// Exercise: Debug for Cell
impl<T: Copy + std::fmt::Debug> std::fmt::Debug for MyCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyCell({:?})", self.get())
    }
}

// ============================================================================
// Exercise: Counter using Cell
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

    // Note: &self, not &mut self!
    fn increment(&self) {
        self.count.set(self.count.get() + 1);
    }

    fn get(&self) -> usize {
        self.count.get()
    }
}

// ============================================================================
// Demo
// ============================================================================

fn _01_basic_get_set() {
    println!("--- Basic get/set ---");
    let cell = MyCell::new(5);
    println!("Initial value: {}", cell.get());
    cell.set(10);
    println!("After set(10): {}", cell.get());
}

fn _02_shared_ref_mutation() {
    println!("\n--- Shared reference mutation ---");
    let cell = MyCell::new(0);
    let ref1 = &cell;
    let ref2 = &cell;
    // Both references can "mutate" through set
    ref1.set(1);
    println!("After ref1.set(1): {}", cell.get());
    ref2.set(2);
    println!("After ref2.set(2): {}", cell.get());
}

fn _03_replace() {
    println!("\n--- Replace ---");
    let cell = MyCell::new(String::from("hello"));
    let old = cell.replace(String::from("world"));
    println!("Old value: {}", old);
    println!("New value: {:?}", unsafe { &*cell.as_ptr() });
}

fn _04_take() {
    println!("\n--- Take ---");
    let cell = MyCell::new(42i32);
    let taken = cell.take();
    println!("Taken: {}", taken);
    println!("Cell now contains: {}", cell.get()); // 0 (default)
}

fn _05_swap() {
    println!("\n--- Swap ---");
    let a = MyCell::new(1);
    let b = MyCell::new(2);
    println!("Before swap: a={}, b={}", a.get(), b.get());
    a.swap(&b);
    println!("After swap: a={}, b={}", a.get(), b.get());
}

fn _06_update() {
    println!("\n--- Update ---");
    let cell = MyCell::new(5);
    cell.update(|x| x * 2);
    println!("After update(|x| x * 2): {}", cell.get());
}

fn _07_counter() {
    println!("\n--- Counter (practical example) ---");
    let counter = Counter::new();
    println!("Initial count: {}", counter.get());

    // Multiple shared references can all increment
    let r1 = &counter;
    let r2 = &counter;
    let r3 = &counter;

    r1.increment();
    r2.increment();
    r3.increment();

    println!("After 3 increments: {}", counter.get());
}

fn _08_clone() {
    println!("\n--- Clone ---");
    let original = MyCell::new(42);
    let cloned = original.clone();
    original.set(100);
    println!("Original: {}", original.get());
    println!("Cloned (unchanged): {}", cloned.get());
}

fn _09_debug() {
    println!("\n--- Debug ---");
    let cell = MyCell::new(99);
    println!("Debug output: {:?}", cell);
}

fn _10_into_inner() {
    println!("\n--- into_inner ---");
    let cell = MyCell::new(String::from("owned"));
    let value = cell.into_inner();
    println!("Consumed cell, got: {}", value);
    // cell.get(); // Error: cell has been moved
}

fn _11_safety() {
    println!("\n--- Why Cell is safe ---");
    let cell = MyCell::new(vec![1, 2, 3]);
    // We can't get a reference to the inner Vec, only replace it
    let old_vec = cell.replace(vec![4, 5, 6]);
    println!("Old vec: {:?}", old_vec);
    // If we could get &Vec, and then call set(), we'd have a dangling reference
    // Cell prevents this by design!
}

fn _12_cell_in_structs() {
    println!("\n--- Cell in structs ---");
    struct Config {
        max_retries: MyCell<u32>,
        timeout_ms: MyCell<u32>,
        debug_mode: MyCell<bool>,
    }

    let config = Config {
        max_retries: MyCell::new(3),
        timeout_ms: MyCell::new(1000),
        debug_mode: MyCell::new(false),
    };

    // Can modify through shared reference
    let cfg = &config;
    cfg.debug_mode.set(true);
    cfg.max_retries.set(5);

    println!("Config: retries={}, timeout={}ms, debug={}",
        config.max_retries.get(),
        config.timeout_ms.get(),
        config.debug_mode.get()
    );
}

fn main() {
    println!("=== MyCell Demo ===\n");

    _01_basic_get_set();
    _02_shared_ref_mutation();
    _03_replace();
    _04_take();
    _05_swap();
    _06_update();
    _07_counter();
    _08_clone();
    _09_debug();
    _10_into_inner();
    _11_safety();
    _12_cell_in_structs();

    println!("\n=== End Demo ===");
}