//! Chapter 6: RefCell - Exercises
//!
//! Complete the TODO items to practice using RefCell0 for runtime borrow checking.
//! Run with: cargo run --example refcell

#![allow(unused)]

#[macro_use]
mod common;

use rustlib::refcell::RefCell0;

// ============================================================================
// Exercises - Replace variables with TODOs with the correct RefCell0 operations
// ============================================================================

fn _01_new_and_borrow() {
    let cell = RefCell0::new(String::from("hello"));
    let value = String::new(); // TODO: borrow from cell and clone the value

    assert_eq!(value, "hello");
}

fn _02_borrow_mut() {
    let cell = RefCell0::new(String::from("hello"));
    // TODO: borrow_mut and push " world" to the string

    assert_eq!(*cell.borrow(), "hello world");
}

fn _03_multiple_borrows() {
    let cell = RefCell0::new(vec![1, 2, 3]);

    // TODO: create three immutable borrows (r1, r2, r3)
    let r1 = cell.borrow();
    let r2 = cell.borrow();
    let r3 = cell.borrow();

    assert_eq!(*r1, vec![1, 2, 3]);
    assert_eq!(*r2, vec![1, 2, 3]);
    assert_eq!(*r3, vec![1, 2, 3]);
    // Multiple immutable borrows can coexist!
}

fn _04_scoped_borrows() {
    let cell = RefCell0::new(10);

    let value = {
        let borrowed = cell.borrow();
        *borrowed
    }; // borrowed dropped here

    // TODO: borrow_mut and multiply by 2

    assert_eq!(*cell.borrow(), 20);
}

fn _05_try_borrow() {
    let cell = RefCell0::new(42);
    let _guard = cell.borrow_mut(); // Hold a mutable borrow

    // TODO: use try_borrow (should fail)
    let result = cell.try_borrow();

    assert!(result.is_err());
}

fn _06_try_borrow_mut() {
    let cell = RefCell0::new(42);
    let _guard1 = cell.borrow(); // Hold an immutable borrow

    // TODO: use try_borrow_mut (should fail)
    let result = cell.try_borrow_mut();

    assert!(result.is_err());
}

fn _07_replace() {
    let cell = RefCell0::new(String::from("old"));
    let old = String::new(); // TODO: replace with "new" using replace()

    assert_eq!(old, "old");
    assert_eq!(*cell.borrow(), "new");
}

fn _08_swap() {
    let a = RefCell0::new(1);
    let b = RefCell0::new(2);

    // TODO: swap a and b

    assert_eq!(*a.borrow(), 2);
    assert_eq!(*b.borrow(), 1);
}

fn _09_into_inner() {
    let cell = RefCell0::new(String::from("owned"));
    let value = String::new(); // TODO: consume cell using into_inner

    assert_eq!(value, "owned");
    // cell is no longer valid here
}

fn _10_clone() {
    let cell1 = RefCell0::new(vec![1, 2, 3]);
    let cell2: RefCell0<Vec<i32>> = RefCell0::new(vec![]); // TODO: clone cell1

    cell1.borrow_mut().push(4);

    assert_eq!(*cell1.borrow(), vec![1, 2, 3, 4]);
    assert_eq!(*cell2.borrow(), vec![1, 2, 3]); // Independent copy
}

fn _11_get_mut() {
    let mut cell = RefCell0::new(5);

    // TODO: use get_mut to add 10 to the value
    // Hint: get_mut returns &mut T and requires &mut self

    assert_eq!(*cell.borrow(), 15);
}

// ============================================================================
// Real-world Demo: Shared Mutable State
// ============================================================================

struct DataStore {
    data: RefCell0<Vec<String>>,
}

impl DataStore {
    fn new() -> Self {
        DataStore {
            data: RefCell0::new(Vec::new()),
        }
    }

    // Note: takes &self, not &mut self!
    fn add(&self, item: String) {
        self.data.borrow_mut().push(item);
    }

    fn count(&self) -> usize {
        self.data.borrow().len()
    }

    fn get(&self, index: usize) -> Option<String> {
        self.data.borrow().get(index).cloned()
    }
}

fn _12_data_store() {
    let store: DataStore = DataStore::new(); // TODO: create new DataStore

    // Multiple shared references can all mutate
    let r1 = &store;
    let r2 = &store;
    let r3 = &store;

    // TODO: use r1, r2, r3 to add items
    r1.add(String::from("first"));
    r2.add(String::from("second"));
    r3.add(String::from("third"));

    assert_eq!(store.count(), 3);
    assert_eq!(store.get(0), Some(String::from("first")));
}

// ============================================================================
// Real-world Demo: Read-then-Write Pattern
// ============================================================================

struct Counter {
    value: RefCell0<i32>,
}

impl Counter {
    fn new(initial: i32) -> Self {
        Counter {
            value: RefCell0::new(initial),
        }
    }

    fn increment_by_current(&self) {
        // Read current value
        let current = *self.value.borrow();
        // Borrow dropped, now safe to mutate
        *self.value.borrow_mut() += current;
    }

    fn get(&self) -> i32 {
        *self.value.borrow()
    }
}

fn _13_counter() {
    let counter = Counter::new(5);

    let initial = 0; // TODO: get initial value
    // TODO: call increment_by_current
    let final_value = 0; // TODO: get final value

    assert_eq!(initial, 5);
    assert_eq!(final_value, 10); // 5 + 5 = 10
}

// ============================================================================
// Real-world Demo: Lazy Computation with Cache
// ============================================================================

struct LazyValue<T> {
    value: RefCell0<Option<T>>,
}

impl<T> LazyValue<T> {
    fn new() -> Self {
        LazyValue {
            value: RefCell0::new(None),
        }
    }

    fn get_or_init<F: FnOnce() -> T>(&self, f: F) -> T
    where
        T: Clone,
    {
        // Check if we have a cached value
        if let Some(ref v) = *self.value.borrow() {
            return v.clone();
        }

        // Compute and cache
        let computed = f();
        *self.value.borrow_mut() = Some(computed.clone());
        computed
    }

    fn clear(&self) {
        *self.value.borrow_mut() = None;
    }
}

fn _14_lazy_value() {
    let lazy: LazyValue<i32> = LazyValue::new(); // TODO: create new LazyValue

    let mut call_count = 0;
    let expensive_fn = || {
        call_count += 1;
        42
    };

    let result1 = 0; // TODO: get value using get_or_init
    let result2 = 0; // TODO: get value again (should be cached)

    assert_eq!(result1, 42);
    assert_eq!(result2, 42);
    assert_eq!(call_count, 1); // Only called once!

    // TODO: clear the cache

    let result3 = lazy.get_or_init(|| 99);
    assert_eq!(result3, 99);
}

// ============================================================================
// Advanced: Graph with Cycles (Preview for Rc + RefCell)
// ============================================================================

#[derive(Debug)]
struct Node {
    value: i32,
    neighbors: RefCell0<Vec<usize>>, // Store indices instead of actual nodes
}

impl Node {
    fn new(value: i32) -> Self {
        Node {
            value,
            neighbors: RefCell0::new(Vec::new()),
        }
    }

    fn add_neighbor(&self, index: usize) {
        self.neighbors.borrow_mut().push(index);
    }

    fn neighbor_count(&self) -> usize {
        self.neighbors.borrow().len()
    }
}

fn _15_graph() {
    // Simple graph: node 0 -> node 1 -> node 2
    let nodes = vec![Node::new(0), Node::new(1), Node::new(2)];

    // TODO: add edges
    nodes[0].add_neighbor(1); // 0 -> 1
    nodes[1].add_neighbor(2); // 1 -> 2
    nodes[2].add_neighbor(0); // 2 -> 0 (cycle!)

    assert_eq!(nodes[0].neighbor_count(), 1);
    assert_eq!(nodes[1].neighbor_count(), 1);
    assert_eq!(nodes[2].neighbor_count(), 1);
}

// ============================================================================
// Pitfall Demo: Holding Borrows Too Long (Uncomment to see panic)
// ============================================================================

fn _16_pitfall_demo() {
    let cell = RefCell0::new(vec![1, 2, 3]);

    // This would panic:
    // let borrowed = cell.borrow();
    // cell.borrow_mut().push(4); // PANIC! Can't borrow_mut while borrowed

    // Correct way:
    {
        let borrowed = cell.borrow();
        println!("Current: {:?}", *borrowed);
    } // borrowed dropped here

    cell.borrow_mut().push(4); // Now safe!

    assert_eq!(*cell.borrow(), vec![1, 2, 3, 4]);
}

// ============================================================================
// Pitfall Demo: Storing Borrows in Collections
// ============================================================================

fn _17_pitfall_storing_borrows() {
    let cell = RefCell0::new(0);

    // BAD: Accumulating borrows in a Vec
    // Uncomment to see the panic:
    // let mut borrows = Vec::new();
    // for _ in 0..5 {
    //     borrows.push(cell.borrow()); // Each borrow is moved into Vec
    // }
    // cell.borrow_mut(); // PANIC! All 5 borrows are still alive in the Vec

    // GOOD: Don't store guards
    for _ in 0..5 {
        let borrowed = cell.borrow();
        println!("Value: {}", *borrowed);
        // borrowed drops at end of iteration
    }

    *cell.borrow_mut() = 42; // Safe!
    assert_eq!(*cell.borrow(), 42);
}

// ============================================================================
// Pitfall Demo: Recursive Borrowing
// ============================================================================

fn _18_pitfall_recursive() {
    let cell = RefCell0::new(5);

    fn recursive_bad(cell: &RefCell0<i32>, depth: i32) {
        if depth == 0 {
            return;
        }

        // BAD: Holding borrow across recursive call
        let _borrowed = cell.borrow();
        recursive_bad(cell, depth - 1);
        if depth == 1 {
            cell.borrow_mut(); // PANIC! Parent call still holds borrow
        }
    }

    fn recursive_good(cell: &RefCell0<i32>, depth: i32) {
        if depth == 0 {
            return;
        }

        // GOOD: Borrow and drop within each call
        {
            let borrowed = cell.borrow();
            println!("Depth {}: {}", depth, *borrowed);
        } // dropped here

        recursive_good(cell, depth - 1);

        // Safe to mutate after recursion
        if depth == 1 {
            *cell.borrow_mut() += 1;
        }
    }

    recursive_good(&cell, 3);
    assert_eq!(*cell.borrow(), 6); // 5 + 1
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    run_all![
        "RefCell0",
        _01_new_and_borrow,
        _02_borrow_mut,
        _03_multiple_borrows,
        _04_scoped_borrows,
        _05_try_borrow,
        _06_try_borrow_mut,
        _07_replace,
        _08_swap,
        _09_into_inner,
        _10_clone,
        _11_get_mut,
        _12_data_store,
        _13_counter,
        _14_lazy_value,
        _15_graph,
        _16_pitfall_demo,
        _17_pitfall_storing_borrows,
        _18_pitfall_recursive,
    ];
}
