# Patterns & Pitfalls

## Common Patterns

### Pattern 1: Graph Structure with Adjacency List

Build a graph where nodes can have edges to other nodes:

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Node {
    id: usize,
    edges: RefCell<Vec<Rc<Node>>>,
}

impl Node {
    fn new(id: usize) -> Rc<Node> {
        Rc::new(Node {
            id,
            edges: RefCell::new(vec![]),
        })
    }

    fn add_edge(&self, target: Rc<Node>) {
        self.edges.borrow_mut().push(target);
    }

    fn neighbors(&self) -> Vec<Rc<Node>> {
        // Clone the Vec (clones each Rc inside, incrementing their reference counts)
        // The returned Vec owns these Rc clones until the caller drops it
        self.edges.borrow().clone()
    }
}

// Usage: Build a simple graph
let node1 = Node::new(1);
let node2 = Node::new(2);
let node3 = Node::new(3);

node1.add_edge(Rc::clone(&node2));
node1.add_edge(Rc::clone(&node3));
node2.add_edge(Rc::clone(&node3));

// Traverse the graph
for neighbor in node1.neighbors() {
    println!("Node {} connects to Node {}", node1.id, neighbor.id);
}
```

**Note:** This creates strong references only, which can leak if there are cycles. For graphs with cycles, use `Weak` for some edges (e.g., back edges).

### Pattern 2: Doubly-Linked List

A list you can traverse in both directions:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    value: i32,
    next: RefCell<Option<Rc<Node>>>,      // Strong forward link
    prev: RefCell<Weak<Node>>,            // Weak backward link
}

impl Node {
    fn new(value: i32) -> Rc<Node> {
        Rc::new(Node {
            value,
            next: RefCell::new(None),
            prev: RefCell::new(Weak::new()),
        })
    }

    fn append(current: &Rc<Node>, next: &Rc<Node>) {
        *next.prev.borrow_mut() = Rc::downgrade(current);
        *current.next.borrow_mut() = Some(Rc::clone(next));
    }
}

// Usage
let head = Node::new(1);
let second = Node::new(2);
let third = Node::new(3);

Node::append(&head, &second);
Node::append(&second, &third);

// Navigate forward
if let Some(next) = head.next.borrow().as_ref() {
    println!("After head: {}", next.value); // 2
}

// Navigate backward
if let Some(prev) = third.prev.borrow().upgrade() {
    println!("Before third: {}", prev.value); // 2
}
```

**Key insight:** Forward links use `Rc` (ownership), backward links use `Weak` (non-owning).

### Pattern 3: Observer Pattern

An observable that notifies multiple observers:

```rust
use std::rc::Rc;
use std::cell::RefCell;

trait Observer {
    fn notify(&self, event: &str);
}

struct Observable {
    observers: RefCell<Vec<Rc<dyn Observer>>>,
}

impl Observable {
    fn new() -> Self {
        Observable {
            observers: RefCell::new(vec![]),
        }
    }

    fn subscribe(&self, observer: Rc<dyn Observer>) {
        self.observers.borrow_mut().push(observer);
    }

    fn notify_all(&self, event: &str) {
        for observer in self.observers.borrow().iter() {
            observer.notify(event);
        }
    }
}

// Concrete observer
struct Logger {
    name: String,
}

impl Observer for Logger {
    fn notify(&self, event: &str) {
        println!("[{}] Received event: {}", self.name, event);
    }
}

// Usage
let observable = Observable::new();

let logger1 = Rc::new(Logger {
    name: String::from("Logger1"),
});
let logger2 = Rc::new(Logger {
    name: String::from("Logger2"),
});

observable.subscribe(Rc::clone(&logger1));
observable.subscribe(Rc::clone(&logger2));

observable.notify_all("UserLoggedIn");
// Output:
// [Logger1] Received event: UserLoggedIn
// [Logger2] Received event: UserLoggedIn
```

**Why this works:** Observers are shared via `Rc` (multiple parts can hold references to the same observer). The observable mutates its list through `RefCell`.

### Pattern 4: Shared Cache with Updates

Multiple readers with occasional updates:

```rust
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

struct Cache {
    data: RefCell<HashMap<String, String>>,
}

impl Cache {
    fn new() -> Rc<Cache> {
        Rc::new(Cache {
            data: RefCell::new(HashMap::new()),
        })
    }

    fn get(&self, key: &str) -> Option<String> {
        self.data.borrow().get(key).cloned()
    }

    fn set(&self, key: String, value: String) {
        self.data.borrow_mut().insert(key, value);
    }
}

// Multiple components share the cache
let cache = Cache::new();
let reader1 = Rc::clone(&cache);
let reader2 = Rc::clone(&cache);
let writer = Rc::clone(&cache);

// Writer updates cache
writer.set(String::from("user:123"), String::from("Alice"));

// Readers can access
if let Some(name) = reader1.get("user:123") {
    println!("Reader1 sees: {}", name);
}
if let Some(name) = reader2.get("user:123") {
    println!("Reader2 sees: {}", name);
}
```

**Important:** This is single-threaded only! For multi-threaded caching, use `Arc<Mutex<HashMap<K, V>>>` (covered in Chapter 14).

## Pitfalls

### Pitfall 1: Creating Accidental Cycles

**BAD:** Both directions using strong references:

```rust
use std::rc::Rc;
use std::cell::RefCell;

struct Node {
    value: i32,
    next: RefCell<Option<Rc<Node>>>,
    prev: RefCell<Option<Rc<Node>>>,  // ❌ BAD: Strong reference both ways!
}

let node_a = Rc::new(Node {
    value: 1,
    next: RefCell::new(None),
    prev: RefCell::new(None),
});

let node_b = Rc::new(Node {
    value: 2,
    next: RefCell::new(None),
    prev: RefCell::new(None),
});

// Create bidirectional strong references - CYCLE!
*node_a.next.borrow_mut() = Some(Rc::clone(&node_b));
*node_b.prev.borrow_mut() = Some(Rc::clone(&node_a));  // Memory leak! 💀

// Counts: node_a = 2, node_b = 2
// When they drop: counts go to 1, never reach 0
// Memory leaked!
```

**FIX:** Use `Weak` for one direction:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    value: i32,
    next: RefCell<Option<Rc<Node>>>,    // Strong forward
    prev: RefCell<Weak<Node>>,          // ✅ Weak backward
}

let node_a = Rc::new(Node {
    value: 1,
    next: RefCell::new(None),
    prev: RefCell::new(Weak::new()),
});

let node_b = Rc::new(Node {
    value: 2,
    next: RefCell::new(None),
    prev: RefCell::new(Weak::new()),
});

// One strong, one weak - NO CYCLE!
*node_a.next.borrow_mut() = Some(Rc::clone(&node_b));
*node_b.prev.borrow_mut() = Rc::downgrade(&node_a);  // ✅ Weak reference

// When they drop, no cycle, memory is freed properly
```

**The rule:** Pick an ownership direction. Use `Rc` for ownership, `Weak` for back-references.

### Pitfall 2: Borrow Panics at Runtime

**BAD:** Holding a borrow while trying to borrow mutably:

```rust
use std::rc::Rc;
use std::cell::RefCell;

let data = Rc::new(RefCell::new(vec![1, 2, 3]));
let data2 = Rc::clone(&data);

// Borrow immutably
let borrowed = data.borrow();
println!("Length: {}", borrowed.len());

// Try to borrow mutably through another Rc
// data2.borrow_mut().push(4);  // 💥 PANIC! Already borrowed immutably

// The borrow from data is still active!
```

**Why it panics:** Because the `RefCell` is shared between `data` and `data2` (via `Rc`), both `Rc` instances point to the same `RefCell`. When `data.borrow()` is active, trying `data2.borrow_mut()` violates the borrowing rules - you can't have both immutable and mutable borrows of the same `RefCell` at the same time.

**FIX 1:** Drop the borrow explicitly:

```rust
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
let data2 = Rc::clone(&data);

let borrowed = data.borrow();
println!("Length: {}", borrowed.len());
drop(borrowed);  // ✅ Drop the borrow

data2.borrow_mut().push(4);  // Now safe!
```

**FIX 2:** Use a shorter scope:

```rust
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
let data2 = Rc::clone(&data);

{
    let borrowed = data.borrow();
    println!("Length: {}", borrowed.len());
}  // ✅ Borrow dropped here

data2.borrow_mut().push(4);  // Now safe!
```

**FIX 3:** Don't store the borrow (best approach):

```rust
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
let data2 = Rc::clone(&data);

println!("Length: {}", data.borrow().len());  // ✅ Borrow dropped immediately
data2.borrow_mut().push(4);  // Now safe!
```

**FIX 4:** Use `try_borrow_mut` to handle gracefully:

```rust
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
let data2 = Rc::clone(&data);

let borrowed = data.borrow();
println!("Length: {}", borrowed.len());

// Try to borrow mutably - returns Err instead of panicking
match data2.try_borrow_mut() {
    Ok(mut b) => {
        b.push(4);
        println!("Successfully modified");
    }
    Err(_) => {
        println!("Can't modify right now, already borrowed");
    }
}
```

### Pitfall 3: Weak Upgrade Failures

**BAD:** Assuming upgrade always succeeds:

```rust
use std::rc::{Rc, Weak};

let strong = Rc::new(42);
let weak = Rc::downgrade(&strong);

drop(strong);  // Value is freed!

let value = weak.upgrade().unwrap();  // 💥 PANIC! upgrade() returns None
```

**FIX:** Always check if upgrade succeeds:

```rust
use std::rc::{Rc, Weak};

let strong = Rc::new(42);
let weak = Rc::downgrade(&strong);

drop(strong);

// ✅ Check before using
if let Some(value) = weak.upgrade() {
    println!("Value: {}", *value);
} else {
    println!("Value has been dropped");
}
```

**Practical example:** Tree traversal with parent pointers:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,
}

fn print_parent_value(node: &Node) {
    match node.parent.borrow().upgrade() {
        Some(parent) => {
            println!("Parent value: {}", parent.value);
        }
        None => {
            println!("No parent (root node or parent dropped)");
        }
    }
}
```

