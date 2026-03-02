//! Chapter 8: Rc0 + RefCell0 - Exercises
//!
//! Complete the TODO items to practice using Rc0<RefCell0<T>> for shared mutable state.
//! Run with: cargo run --example 08_rc_refcell

#![allow(unused)]

#[macro_use]
mod common;

use rustlib::rc::{Rc0, Weak0};
use rustlib::refcell::{BorrowError, RefCell0};

// ============================================================================
// Exercises - Replace variables with TODOs with the correct operations
// ============================================================================

fn _01_create_and_access() {
    // Create an Rc0<RefCell0<i32>> to allow shared ownership with interior mutability
    let data = Rc0::new(RefCell0::new(42));

    let value = *data.borrow();

    assert_eq!(value, 42);
}

fn _02_mutate_through_rc() {
    let data = Rc0::new(RefCell0::new(10));

    *data.borrow_mut() += 5;

    assert_eq!(*data.borrow(), 15);
}

fn _03_multiple_owners_mutate() {
    let data = Rc0::new(RefCell0::new(0));

    // Create multiple owners
    let owner1 = Rc0::clone(&data);
    let owner2 = Rc0::clone(&data);
    let owner3 = Rc0::clone(&data);

    *owner1.borrow_mut() += 10;
    *owner2.borrow_mut() += 20;
    *owner3.borrow_mut() += 30;

    assert_eq!(*data.borrow(), 60);
    assert_eq!(Rc0::strong_count(&data), 4); // data + 3 owners
}

fn _03b_pattern_comparison() {
    // Pattern 1: Rc0<RefCell0<T>> - Multiple owners can mutate THE SAME value
    let counter = Rc0::new(RefCell0::new(0));
    let c1 = Rc0::clone(&counter);
    let c2 = Rc0::clone(&counter);

    *c1.borrow_mut() += 1;
    *c2.borrow_mut() += 1;

    assert_eq!(*counter.borrow(), 2); // Both mutations affect the SAME value

    // Pattern 2: RefCell0<Rc0<T>> - One owner can CHANGE WHICH Rc0 it points to
    let value_a = Rc0::new(10);
    let value_b = Rc0::new(20);

    let pointer = RefCell0::new(Rc0::clone(&value_a)); // Points to value_a initially

    assert_eq!(**pointer.borrow(), 10); // Pointing to value_a

    *pointer.borrow_mut() = Rc0::clone(&value_b);

    assert_eq!(**pointer.borrow(), 20); // Now pointing to value_b

    // Key difference:
    // - Rc0<RefCell0<T>>: Multiple owners -> mutate the value inside
    // - RefCell0<Rc0<T>>: One owner -> change which Rc0 you're pointing to
}

fn _04_clone_semantics() {
    let data = Rc0::new(RefCell0::new(vec![1, 2, 3]));

    let rc_clone: Rc0<RefCell0<Vec<i32>>> = Rc0::clone(&data);

    let data_clone: Vec<i32> = data.borrow().clone();

    data.borrow_mut().push(4);

    // rc_clone sees the change (both point to same RefCell0)
    assert_eq!(*rc_clone.borrow(), vec![1, 2, 3, 4]);

    // data_clone is independent (it's a separate Vec)
    assert_eq!(data_clone, vec![1, 2, 3]);
}

fn _05_reference_counts() {
    let data = Rc0::new(RefCell0::new(String::from("hello")));

    let strong1 = Rc0::strong_count(&data);
    let weak1 = Rc0::weak_count(&data);

    let weak = Rc0::downgrade(&data);

    let strong2 = Rc0::strong_count(&data);
    let weak2 = Rc0::weak_count(&data);

    assert_eq!(strong1, 1);
    assert_eq!(weak1, 0);
    assert_eq!(strong2, 1);
    assert_eq!(weak2, 1);
}

fn _06_create_weak() {
    let data = Rc0::new(RefCell0::new(100));

    let weak: Weak0<RefCell0<i32>> = Rc0::downgrade(&data);

    assert_eq!(Rc0::strong_count(&data), 1);
    assert_eq!(Rc0::weak_count(&data), 1);
}

fn _07_upgrade_weak_success() {
    let data = Rc0::new(RefCell0::new(42));
    let weak = Rc0::downgrade(&data);

    if let Some(strong) = weak.upgrade() {
        assert_eq!(*strong.borrow(), 42);
    } else {
        panic!("upgrade should succeed");
    }
}

fn _08_upgrade_weak_after_drop() {
    let weak = {
        let data = Rc0::new(RefCell0::new(42));
        Rc0::downgrade(&data)
    }; // data dropped here

    let result: Option<Rc0<RefCell0<i32>>> = weak.upgrade();

    assert!(result.is_none());
}

fn _09_weak_with_refcell() {
    let data = Rc0::new(RefCell0::new(vec![1, 2, 3]));
    let weak = Rc0::downgrade(&data);

    if let Some(strong) = weak.upgrade() {
        strong.borrow_mut().push(4);
    }

    assert_eq!(*data.borrow(), vec![1, 2, 3, 4]);
}

fn _10_try_borrow() {
    let data = Rc0::new(RefCell0::new(42));

    let _guard = data.borrow_mut(); // Hold a mutable borrow

    let result: Result<_, BorrowError> = data.try_borrow();

    assert!(result.is_err());
}

fn _11_replace_contents() {
    let data = Rc0::new(RefCell0::new(String::from("old")));

    let old = data.replace(String::from("new"));

    assert_eq!(old, "old");
    assert_eq!(*data.borrow(), "new");
}

// ============================================================================
// Real-world Demo: Graph Structure with Adjacency Lists
// ============================================================================

#[derive(Debug)]
struct GraphNode {
    value: i32,
    neighbors: RefCell0<Vec<Rc0<GraphNode>>>,
}

impl GraphNode {
    fn new(value: i32) -> Rc0<Self> {
        Rc0::new(GraphNode {
            value,
            neighbors: RefCell0::new(Vec::new()),
        })
    }

    fn add_neighbor(&self, neighbor: Rc0<GraphNode>) {
        self.neighbors.borrow_mut().push(neighbor);
    }

    fn neighbor_count(&self) -> usize {
        self.neighbors.borrow().len()
    }

    fn neighbors(&self) -> Vec<Rc0<GraphNode>> {
        self.neighbors.borrow().clone()
    }
}

fn _12_graph_structure() {
    // Create nodes
    let node_a = GraphNode::new(1);
    let node_b = GraphNode::new(2);
    let node_c = GraphNode::new(3);

    node_a.add_neighbor(Rc0::clone(&node_b));
    node_b.add_neighbor(Rc0::clone(&node_c));

    assert_eq!(node_a.neighbor_count(), 1);
    assert_eq!(node_b.neighbor_count(), 1);
    assert_eq!(node_c.neighbor_count(), 0);

    // Check we can traverse
    let a_neighbors = node_a.neighbors();
    assert_eq!(a_neighbors[0].value, 2);
}

// ============================================================================
// Real-world Demo: Tree with Parent Pointers
// ============================================================================

#[derive(Debug)]
struct TreeNode {
    value: i32,
    parent: RefCell0<Weak0<TreeNode>>,
    children: RefCell0<Vec<Rc0<TreeNode>>>,
}

impl TreeNode {
    fn new(value: i32) -> Rc0<Self> {
        Rc0::new(TreeNode {
            value,
            parent: RefCell0::new(Weak0::new()),
            children: RefCell0::new(Vec::new()),
        })
    }

    fn add_child(parent: &Rc0<TreeNode>, child: Rc0<TreeNode>) {
        // Set child's parent to weak reference
        *child.parent.borrow_mut() = Rc0::downgrade(parent);
        // Add child to parent's children
        parent.children.borrow_mut().push(child);
    }

    fn parent(&self) -> Option<Rc0<TreeNode>> {
        self.parent.borrow().upgrade()
    }

    fn child_count(&self) -> usize {
        self.children.borrow().len()
    }
}

fn _13_tree_with_parent() {
    let root = TreeNode::new(1);
    let child1 = TreeNode::new(2);
    let child2 = TreeNode::new(3);
    let grandchild = TreeNode::new(4);

    TreeNode::add_child(&root, Rc0::clone(&child1));
    TreeNode::add_child(&root, Rc0::clone(&child2));
    TreeNode::add_child(&child1, Rc0::clone(&grandchild));

    // Verify structure
    assert_eq!(root.child_count(), 2);
    assert_eq!(child1.child_count(), 1);

    // Navigate from child to parent
    assert_eq!(child1.parent().unwrap().value, 1);
    assert_eq!(grandchild.parent().unwrap().value, 2);

    // Root has no parent
    assert!(root.parent().is_none());

    // Reference counts
    assert_eq!(Rc0::strong_count(&root), 1); // Only root owns it
    assert_eq!(Rc0::weak_count(&root), 2);   // Two children point to it weakly
}

// ============================================================================
// Real-world Demo: Observer Pattern
// ============================================================================

trait Observer {
    fn notify(&self, event: &str);
}

struct Logger {
    name: String,
}

impl Logger {
    fn new(name: &str) -> Self {
        Logger {
            name: name.to_string(),
        }
    }
}

impl Observer for Logger {
    fn notify(&self, event: &str) {
        println!("[{}] Event: {}", self.name, event);
    }
}

struct Observable<T> {
    observers: RefCell0<Vec<Rc0<T>>>,
}

impl<T: Observer> Observable<T> {
    fn new() -> Self {
        Observable {
            observers: RefCell0::new(Vec::new()),
        }
    }

    fn subscribe(&self, observer: Rc0<T>) {
        self.observers.borrow_mut().push(observer);
    }

    fn notify_all(&self, event: &str) {
        for observer in self.observers.borrow().iter() {
            observer.notify(event);
        }
    }
}

fn _14_observer_pattern() {
    let observable: Observable<Logger> = Observable::new();

    let logger1 = Rc0::new(Logger::new("Logger1"));
    let logger2 = Rc0::new(Logger::new("Logger2"));

    observable.subscribe(Rc0::clone(&logger1));
    observable.subscribe(Rc0::clone(&logger2));

    // Notify all observers
    observable.notify_all("data_changed");

    // Both loggers remain valid
    logger1.notify("Still working");
}

// ============================================================================
// Real-world Demo: Shared Cache
// ============================================================================

struct CacheEntry {
    key: String,
    value: String,
    access_count: RefCell0<usize>,
}

impl CacheEntry {
    fn new(key: String, value: String) -> Self {
        CacheEntry {
            key,
            value,
            access_count: RefCell0::new(0),
        }
    }

    fn access(&self) -> String {
        *self.access_count.borrow_mut() += 1;
        self.value.clone()
    }

    fn count(&self) -> usize {
        *self.access_count.borrow()
    }
}

struct Cache {
    entries: RefCell0<Vec<Rc0<CacheEntry>>>,
}

impl Cache {
    fn new() -> Self {
        Cache {
            entries: RefCell0::new(Vec::new()),
        }
    }

    fn insert(&self, key: String, value: String) {
        let entry = Rc0::new(CacheEntry::new(key, value));
        self.entries.borrow_mut().push(entry);
    }

    fn get(&self, key: &str) -> Option<Rc0<CacheEntry>> {
        self.entries
            .borrow()
            .iter()
            .find(|e| e.key == key)
            .cloned()
    }
}

fn _15_shared_cache() {
    let cache = Cache::new();

    cache.insert(String::from("user:1"), String::from("Alice"));
    cache.insert(String::from("user:2"), String::from("Bob"));

    // Multiple readers can access the cache
    let reader1 = &cache;
    let reader2 = &cache;

    let entry1 = reader1.get("user:1").unwrap();
    let entry2 = reader2.get("user:1").unwrap();

    // Both point to the same entry
    assert!(Rc0::ptr_eq(&entry1, &entry2));

    // Access the entry multiple times
    assert_eq!(entry1.access(), "Alice");
    assert_eq!(entry2.access(), "Alice");
    assert_eq!(entry1.count(), 2);
}

// ============================================================================
// Pitfall Demo: Accidental Cycle Creation (Memory Leak)
// ============================================================================

#[derive(Debug)]
struct CycleNode {
    value: i32,
    next: RefCell0<Option<Rc0<CycleNode>>>,
}

impl CycleNode {
    fn new(value: i32) -> Rc0<Self> {
        Rc0::new(CycleNode {
            value,
            next: RefCell0::new(None),
        })
    }
}

impl Drop for CycleNode {
    fn drop(&mut self) {
        println!("  Dropping CycleNode with value {}", self.value);
    }
}

fn _16_pitfall_cycle() {
    println!("\n--- Creating a Cycle (Memory Leak) ---");

    // BAD: Creating a cycle causes memory leak
    {
        let node_a = CycleNode::new(1);
        let node_b = CycleNode::new(2);

        // Create cycle: A -> B -> A
        *node_a.next.borrow_mut() = Some(Rc0::clone(&node_b));
        *node_b.next.borrow_mut() = Some(Rc0::clone(&node_a));

        println!("Created cycle: A -> B -> A");
        println!("strong_count(node_a) = {}", Rc0::strong_count(&node_a)); // 2
        println!("strong_count(node_b) = {}", Rc0::strong_count(&node_b)); // 2

        // When we exit this scope:
        // - node_a's stack reference drops, count goes to 1 (b still holds it)
        // - node_b's stack reference drops, count goes to 1 (a still holds it)
        // - Neither reaches 0, so neither gets freed!
        // - Memory is leaked, just like std::mem::forget from Chapter 7
    }

    println!("After scope: NO Drop messages! Memory leaked!\n");

    // GOOD: Use Weak0 to break the cycle
    println!("--- Breaking Cycle with Weak0 ---");

    #[derive(Debug)]
    struct SafeNode {
        value: i32,
        next: RefCell0<Weak0<SafeNode>>, // Weak0 instead of Rc0!
    }

    impl SafeNode {
        fn new(value: i32) -> Rc0<Self> {
            Rc0::new(SafeNode {
                value,
                next: RefCell0::new(Weak0::new()),
            })
        }
    }

    impl Drop for SafeNode {
        fn drop(&mut self) {
            println!("  Dropping SafeNode with value {}", self.value);
        }
    }

    {
        let node_a = SafeNode::new(1);
        let node_b = SafeNode::new(2);

        // Create "cycle" with Weak0: A -> B -weak-> A
        *node_a.next.borrow_mut() = Rc0::downgrade(&node_b);
        *node_b.next.borrow_mut() = Rc0::downgrade(&node_a);

        println!("Created weak cycle");
        println!("strong_count(node_a) = {}", Rc0::strong_count(&node_a)); // 1
        println!("strong_count(node_b) = {}", Rc0::strong_count(&node_b)); // 1
    }

    println!("After scope: Drop messages appear! Memory freed!\n");
}

// ============================================================================
// Pitfall Demo: Borrow Panic from Holding Borrow Too Long
// ============================================================================

fn _17_pitfall_borrow_panic() {
    let data = Rc0::new(RefCell0::new(vec![1, 2, 3]));

    // BAD: Holding borrow across operations
    // This would panic:
    // let borrowed = data.borrow();
    // data.borrow_mut().push(4); // PANIC! Can't borrow_mut while borrowed

    // GOOD: Drop borrow before mutating
    {
        let borrowed = data.borrow();
        println!("Current: {:?}", *borrowed);
    } // borrowed dropped here

    data.borrow_mut().push(4); // Now safe!

    // GOOD: Use try_borrow to avoid panic
    let _guard = data.borrow();

    match data.try_borrow_mut() {
        Ok(_) => panic!("Should not succeed"),
        Err(_) => println!("try_borrow_mut correctly returned Err"),
    }

    assert_eq!(*data.borrow(), vec![1, 2, 3, 4]);
}

// ============================================================================
// Pitfall Demo: Weak0 Upgrade Failure Handling
// ============================================================================

fn _18_pitfall_weak_upgrade() {
    let weak = {
        let data = Rc0::new(RefCell0::new(String::from("temporary")));
        let w = Rc0::downgrade(&data);

        // Can upgrade while data is alive
        assert!(w.upgrade().is_some());

        w
    }; // data dropped here

    // BAD: Not checking upgrade result
    // This would panic:
    // let strong = weak.upgrade().unwrap(); // PANIC! Returns None

    // GOOD: Always check upgrade result
    match weak.upgrade() {
        Some(strong) => {
            println!("Data: {}", *strong.borrow());
        }
        None => {
            println!("Data has been dropped");
        }
    }

    // GOOD: Use if-let pattern
    if let Some(strong) = weak.upgrade() {
        println!("Data: {}", *strong.borrow());
    } else {
        println!("Data no longer available");
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    run_all![
        "Rc0<RefCell0<T>>",
        _01_create_and_access,
        _02_mutate_through_rc,
        _03_multiple_owners_mutate,
        _03b_pattern_comparison,
        _04_clone_semantics,
        _05_reference_counts,
        _06_create_weak,
        _07_upgrade_weak_success,
        _08_upgrade_weak_after_drop,
        _09_weak_with_refcell,
        _10_try_borrow,
        _11_replace_contents,
        _12_graph_structure,
        _13_tree_with_parent,
        _14_observer_pattern,
        _15_shared_cache,
        _16_pitfall_cycle,
        _17_pitfall_borrow_panic,
        _18_pitfall_weak_upgrade,
    ];
}
