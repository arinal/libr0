//! Chapter 7: Rc - Exercises
//!
//! Complete the TODO items to practice using Rc0 for shared ownership.
//! Run with: cargo run --example rc

#![allow(unused)]

#[macro_use]
mod common;

use rustlib::rc::Rc0;

// ============================================================================
// Exercises - Replace variables with TODOs with the correct Rc0 operations
// ============================================================================

fn _01_new_and_clone() {
    let rc1 = Rc0::new(String::from("shared data"));
    let rc2 = Rc0::new(String::new()); // TODO: clone rc1 using Rc0::clone

    assert_eq!(*rc1, "shared data");
    assert_eq!(*rc2, "shared data");
    // Both rc1 and rc2 point to the same data!
}

fn _02_strong_count() {
    let rc1 = Rc0::new(42);
    let count1 = 0; // TODO: get strong_count of rc1

    let rc2 = Rc0::clone(&rc1);
    let count2 = 0; // TODO: get strong_count of rc1 again

    let rc3 = rc1.clone();
    let count3 = 0; // TODO: get strong_count of rc1 again

    assert_eq!(count1, 1);
    assert_eq!(count2, 2);
    assert_eq!(count3, 3);
}

fn _03_deref_coercion() {
    let rc = Rc0::new(String::from("hello world"));

    // Rc<String> can be used as &str through deref coercion
    fn count_words(s: &str) -> usize {
        s.split_whitespace().count()
    }

    let word_count = 0; // TODO: call count_words with rc

    assert_eq!(word_count, 2);
}

fn _04_drop_reduces_count() {
    let rc1 = Rc0::new(100);
    let rc2 = Rc0::clone(&rc1);
    let rc3 = Rc0::clone(&rc1);

    assert_eq!(Rc0::strong_count(&rc1), 3);

    // TODO: drop rc2

    assert_eq!(Rc0::strong_count(&rc1), 2);

    // TODO: drop rc3

    assert_eq!(Rc0::strong_count(&rc1), 1);
}

fn _05_ptr_eq() {
    let rc1 = Rc0::new(42);
    let rc2 = Rc0::clone(&rc1);
    let rc3 = Rc0::new(42); // Same value, different allocation

    let same_alloc = false; // TODO: check if rc1 and rc2 point to same allocation
    let diff_alloc = true; // TODO: check if rc1 and rc3 point to same allocation

    assert_eq!(same_alloc, true);
    assert_eq!(diff_alloc, false);
}

fn _06_get_mut_sole_owner() {
    let mut rc = Rc0::new(vec![1, 2, 3]);

    // When we're the sole owner, we can get mutable access
    if let Some(vec) = Rc0::get_mut(&mut rc) {
        // TODO: push 4 to the vec
    }

    assert_eq!(*rc, vec![1, 2, 3, 4]);
}

fn _07_get_mut_shared() {
    let mut rc1 = Rc0::new(String::from("hello"));
    let rc2 = Rc0::clone(&rc1);

    // Can't get mutable access when shared
    let result = Rc0::get_mut(&mut rc1);

    assert!(result.is_none());

    // TODO: drop rc2

    // Now we can mutate
    if let Some(s) = Rc0::get_mut(&mut rc1) {
        s.push_str(" world");
    }

    assert_eq!(*rc1, "hello world");
}

fn _08_clone_is_cheap() {
    let big_data = Rc0::new(vec![0u8; 1_000_000]); // 1MB

    // Cloning Rc is cheap - just increments a counter
    let clone1: Rc0<Vec<u8>> = Rc0::new(vec![]); // TODO: clone big_data
    let clone2: Rc0<Vec<u8>> = Rc0::new(vec![]); // TODO: clone big_data

    // All three point to the same 1MB allocation
    assert_eq!(Rc0::strong_count(&big_data), 3);
}

fn _09_into_inner() {
    let rc = Rc0::new(String::from("owned"));

    // TODO: This won't compile yet - try_unwrap is not implemented
    // let value = Rc0::try_unwrap(rc).unwrap();
    // assert_eq!(value, "owned");

    // For now, we can only access through deref
    assert_eq!(*rc, "owned");
}

fn _10_clone_semantics() {
    let rc = Rc0::new(vec![1, 2, 3]);

    // Two ways to clone - both work, but Rc0::clone is clearer
    let clone1 = rc.clone(); // Works, but might look expensive
    let clone2 = Rc0::clone(&rc); // ✅ Preferred - clearly cloning the Rc

    assert_eq!(Rc0::strong_count(&rc), 3);
    assert!(Rc0::ptr_eq(&rc, &clone1));
    assert!(Rc0::ptr_eq(&rc, &clone2));
}

fn _11_default() {
    // Rc implements Default when T implements Default
    let rc: Rc0<Vec<i32>> = Rc0::new(vec![]); // TODO: create using Default trait

    assert_eq!(*rc, Vec::<i32>::new());
}

// ============================================================================
// Real-world Demo: Shared Configuration
// ============================================================================

struct Config {
    db_url: String,
    api_key: String,
    max_connections: usize,
}

struct Server {
    config: Rc0<Config>,
}

struct Logger {
    config: Rc0<Config>,
}

struct Database {
    config: Rc0<Config>,
}

impl Server {
    fn new(config: Rc0<Config>) -> Self {
        Server { config }
    }

    fn db_url(&self) -> &str {
        &self.config.db_url
    }
}

impl Logger {
    fn new(config: Rc0<Config>) -> Self {
        Logger { config }
    }

    fn log_startup(&self) {
        println!("Starting with DB: {}", self.config.db_url);
    }
}

impl Database {
    fn new(config: Rc0<Config>) -> Self {
        Database { config }
    }

    fn max_connections(&self) -> usize {
        self.config.max_connections
    }
}

fn _12_shared_config() {
    let config: Rc0<Config> = Rc0::new(Config {
        db_url: String::from("localhost:5432"),
        api_key: String::from("secret123"),
        max_connections: 10,
    }); // TODO: create Rc0 with Config

    let server = Server::new(Rc0::clone(&config));
    let logger = Logger::new(Rc0::clone(&config));
    let db = Database::new(Rc0::clone(&config));

    assert_eq!(server.db_url(), "localhost:5432");
    assert_eq!(db.max_connections(), 10);
    assert_eq!(Rc0::strong_count(&config), 4); // config + 3 components
}

// ============================================================================
// Real-world Demo: Tree with Shared Children
// ============================================================================

#[derive(Debug)]
struct Node {
    value: i32,
    children: Vec<Rc0<Node>>,
}

impl Node {
    fn new(value: i32) -> Self {
        Node {
            value,
            children: Vec::new(),
        }
    }

    fn with_children(value: i32, children: Vec<Rc0<Node>>) -> Self {
        Node { value, children }
    }

    fn child_count(&self) -> usize {
        self.children.len()
    }
}

fn _13_shared_tree() {
    // Create leaf nodes
    let leaf1 = Rc0::new(Node::new(1));
    let leaf2 = Rc0::new(Node::new(2));
    let leaf3 = Rc0::new(Node::new(3));

    // Create parent nodes that share children
    let parent1 = Rc0::new(Node::with_children(
        10,
        vec![Rc0::clone(&leaf1), Rc0::clone(&leaf2)],
    ));

    let parent2 = Rc0::new(Node::with_children(
        20,
        vec![Rc0::clone(&leaf2), Rc0::clone(&leaf3)],
    ));

    // leaf2 is shared between parent1 and parent2!
    assert_eq!(Rc0::strong_count(&leaf1), 2); // leaf1 + parent1
    assert_eq!(Rc0::strong_count(&leaf2), 3); // leaf2 + parent1 + parent2
    assert_eq!(Rc0::strong_count(&leaf3), 2); // leaf3 + parent2

    assert_eq!(parent1.child_count(), 2);
    assert_eq!(parent2.child_count(), 2);
}

// ============================================================================
// Real-world Demo: Functional List (Sharing Tails)
// ============================================================================

#[derive(Debug)]
enum List<T> {
    Cons(T, Rc0<List<T>>),
    Nil,
}

impl<T> List<T> {
    fn new() -> Rc0<Self> {
        Rc0::new(List::Nil)
    }

    fn cons(value: T, tail: Rc0<List<T>>) -> Rc0<Self> {
        Rc0::new(List::Cons(value, tail))
    }

    fn len(&self) -> usize {
        match self {
            List::Nil => 0,
            List::Cons(_, tail) => 1 + tail.len(),
        }
    }
}

fn _14_functional_list() {
    // Create a shared tail: [3, 4, 5]
    let shared_tail = List::cons(
        5,
        List::cons(4, List::cons(3, List::new())),
    );

    // Two lists sharing the same tail
    let list1 = List::cons(2, List::cons(1, Rc0::clone(&shared_tail)));
    let list2 = List::cons(10, List::cons(20, Rc0::clone(&shared_tail)));

    // list1: [1, 2, 3, 4, 5]
    // list2: [20, 10, 3, 4, 5]
    // Both share [3, 4, 5] in memory!

    assert_eq!(list1.len(), 5);
    assert_eq!(list2.len(), 5);
    assert_eq!(Rc0::strong_count(&shared_tail), 3); // shared_tail + list1 + list2
}

// ============================================================================
// Real-world Demo: Caching/Flyweight Pattern
// ============================================================================

use std::collections::HashMap;

struct FontCache {
    fonts: HashMap<String, Rc0<Vec<u8>>>, // Font name -> font data
}

impl FontCache {
    fn new() -> Self {
        FontCache {
            fonts: HashMap::new(),
        }
    }

    fn get(&mut self, name: &str) -> Rc0<Vec<u8>> {
        // Return cached font or load and cache it
        self.fonts
            .entry(name.to_string())
            .or_insert_with(|| {
                // Simulate loading font data
                println!("Loading font: {}", name);
                Rc0::new(vec![0u8; 1000]) // Mock font data
            })
            .clone() // Cheap Rc clone!
    }
}

fn _15_font_cache() {
    let mut cache = FontCache::new();

    let font1 = cache.get("Arial");
    let font2 = cache.get("Arial"); // Returns same Rc, doesn't reload!
    let font3 = cache.get("Times");

    // font1 and font2 share the same allocation
    assert!(Rc0::ptr_eq(&font1, &font2));
    assert!(!Rc0::ptr_eq(&font1, &font3));

    // Arial has 3 references: cache + font1 + font2
    assert_eq!(Rc0::strong_count(&font1), 3);
}

// ============================================================================
// Pitfall Demo: Cloning the Inner Value (Expensive!)
// ============================================================================

fn _16_pitfall_clone_inner() {
    let rc = Rc0::new(vec![1, 2, 3, 4, 5]);

    // BAD: Clones the Vec (expensive)
    let vec_clone = (*rc).clone();

    // GOOD: Clones the Rc (cheap)
    let rc_clone = Rc0::clone(&rc);

    // vec_clone is independent
    assert_eq!(vec_clone, vec![1, 2, 3, 4, 5]);

    // rc_clone shares the same data
    assert!(Rc0::ptr_eq(&rc, &rc_clone));
    assert_eq!(Rc0::strong_count(&rc), 2);
}

// ============================================================================
// Pitfall Demo: Trying to Mutate When Shared
// ============================================================================

fn _17_pitfall_mutation() {
    let mut rc1 = Rc0::new(vec![1, 2, 3]);
    let rc2 = Rc0::clone(&rc1);

    // Can't mutate when shared
    // This would fail:
    // if let Some(vec) = Rc0::get_mut(&mut rc1) {
    //     vec.push(4); // Never runs because strong_count > 1
    // }

    assert!(Rc0::get_mut(&mut rc1).is_none());

    // Drop the clone first
    drop(rc2);

    // Now we can mutate
    if let Some(vec) = Rc0::get_mut(&mut rc1) {
        vec.push(4);
    }

    assert_eq!(*rc1, vec![1, 2, 3, 4]);
}

// ============================================================================
// Pitfall Demo: Demonstrating Memory Leaks
// ============================================================================

fn _18_pitfall_memory_leak() {
    use std::mem::forget;

    // First, let's show normal cleanup
    println!("\n--- Normal Cleanup ---");
    {
        let data = Rc0::new(vec![1, 2, 3, 4, 5]);
        println!("Created Rc, strong_count = {}", Rc0::strong_count(&data));
    } // data dropped here, memory freed ✅
    println!("After scope: memory was freed!");

    // Now let's intentionally leak memory
    println!("\n--- Intentional Memory Leak ---");
    {
        let data = Rc0::new(vec![10, 20, 30, 40, 50]);
        println!("Created Rc, strong_count = {}", Rc0::strong_count(&data));

        // std::mem::forget prevents Drop from running
        forget(data);
        println!("Called forget() - data will NEVER be freed!");
    } // data was NOT dropped, memory leaked! ❌
    println!("After scope: memory is still allocated on heap, leaked forever!");

    // This is exactly what happens with cycles!
    println!("\n--- Why Cycles Leak ---");
    println!("In a cycle:");
    println!("  node_a.next -> node_b (count = 1)");
    println!("  node_b.next -> node_a (count = 1)");
    println!("When you drop the stack variables:");
    println!("  - Both counts stay at 1 (they keep each other alive)");
    println!("  - Neither reaches 0, so neither gets freed");
    println!("  - Memory leaked forever, just like with forget()!");
    println!("\nSolution: Use Weak references (Chapter 13)");
}

// ============================================================================
// Pitfall Demo: Structure That Would Leak With Cycles
// ============================================================================

#[derive(Debug)]
struct CycleNode {
    value: i32,
    next: Option<Rc0<CycleNode>>,
}

impl CycleNode {
    fn new(value: i32) -> Rc0<Self> {
        Rc0::new(CycleNode { value, next: None })
    }
}

impl Drop for CycleNode {
    fn drop(&mut self) {
        println!("  Dropping node with value {}", self.value);
    }
}

fn _19_pitfall_cycle_structure() {
    println!("\n--- Potential Cycle Structure ---");

    // We can create nodes, but can't create cycles without interior mutability
    let node_a = CycleNode::new(1);
    let node_b = CycleNode::new(2);
    let node_c = CycleNode::new(3);

    // This compiles but doesn't create a cycle (all next fields are None)
    println!("Created 3 nodes with next = None");
    println!("strong_count(node_a) = {}", Rc0::strong_count(&node_a));
    println!("strong_count(node_b) = {}", Rc0::strong_count(&node_b));
    println!("strong_count(node_c) = {}", Rc0::strong_count(&node_c));

    // We CANNOT do this because node_a is not mutable:
    // node_a.next = Some(Rc0::clone(&node_b));  // ❌ ERROR: cannot mutate
    // node_b.next = Some(Rc0::clone(&node_c));  // ❌ ERROR: cannot mutate
    // node_c.next = Some(Rc0::clone(&node_a));  // ❌ ERROR: cannot mutate (cycle!)

    println!("\nTo create a cycle, we'd need to mutate 'next' after creation.");
    println!("But Rc only gives us &T, not &mut T!");
    println!("Solution: Rc<RefCell<Node>> - covered in Chapter 13");

    println!("\nDropping nodes (watch the Drop output):");
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    run_all![
        "Rc0",
        _01_new_and_clone,
        _02_strong_count,
        _03_deref_coercion,
        _04_drop_reduces_count,
        _05_ptr_eq,
        _06_get_mut_sole_owner,
        _07_get_mut_shared,
        _08_clone_is_cheap,
        _09_into_inner,
        _10_clone_semantics,
        _11_default,
        _12_shared_config,
        _13_shared_tree,
        _14_functional_list,
        _15_font_cache,
        _16_pitfall_clone_inner,
        _17_pitfall_mutation,
        _18_pitfall_memory_leak,
        _19_pitfall_cycle_structure,
    ];
}