# Chapter 8: Rc + RefCell - Shared Mutable State (Single-Threaded)

## The Problem: Rc Can't Mutate

You learned about `Rc<T>` in Chapter 7 - it gives you shared ownership. Multiple parts of your code can own the same data:

```rust
use std::rc::Rc;

let config = Rc::new(Config { port: 8080 });
let server = Rc::clone(&config);  // ✅ Multiple owners!
let logger = Rc::clone(&config);  // ✅ Everyone can read!
```

But what if you need to **mutate** the shared data?

```rust
let counter = Rc::new(0);
// *counter += 1;  // ❌ ERROR: cannot borrow as mutable
```

**Why it fails:** `Rc<T>` only gives you `&T` (shared reference), never `&mut T`. If multiple owners could get `&mut T`, you'd have multiple mutable references to the same data - a data race!

**Real-world scenarios where you need shared mutable data:**

1. **Graph structures** - Nodes need to add/remove edges to other nodes
2. **Trees with parent pointers** - Children need to update their parent references
3. **Observer pattern** - Observable needs to notify and update observers
4. **Shared cache** - Multiple readers with occasional updates

You need a way to have multiple owners AND mutation. `Rc<T>` alone isn't enough.

## The Solution: Rc<RefCell<T>>

Remember `RefCell<T>` from Chapter 6? It gives you **interior mutability** - the ability to mutate through a shared reference:

```rust
use std::cell::RefCell;

let cell = RefCell::new(5);
*cell.borrow_mut() += 1;  // ✅ Mutate through &RefCell!
```

**Combining them gives you both:**

```rust
use std::rc::Rc;
use std::cell::RefCell;

// Shared ownership + Interior mutability
let counter = Rc::new(RefCell::new(0));

// Clone the Rc to create multiple owners
let counter1 = Rc::clone(&counter);
let counter2 = Rc::clone(&counter);

// All owners can mutate the same data!
*counter1.borrow_mut() += 1;  // ✅ Works!
*counter2.borrow_mut() += 1;  // ✅ Works!

println!("Counter: {}", counter.borrow()); // 2
```

### Memory Layout: Two Levels of Tracking

`Rc<RefCell<T>>` has two levels of tracking in a single allocation:

```
Stack                             Heap

┌───────────┐
│  counter  │                    ┌──────────────────────────────┐
│  (Rc ptr) │───────┬───────────>│ Single RcInner allocation:   │
└───────────┘       │            │  strong_count: 3             │ ← Rc tracks # of owners
┌───────────┐       │            │  weak_count: 0               │
│ counter1  │       │            │                              │
│  (Rc ptr) │───────┤            │  RefCell<i32> (inlined):     │
└───────────┘       │            │    borrow: 0                 │ ← RefCell tracks borrows
┌───────────┐       │            │    value: 2                  │ (after two increments)
│ counter2  │       │            │                              │
│  (Rc ptr) │───────┘            └──────────────────────────────┘
└───────────┘
                                   One shared allocation on heap
```

**Two kinds of tracking:**

1. **Rc level**: Reference counting (how many owners exist)
2. **RefCell level**: Borrow checking (is anyone currently borrowing?)

**Two ways things can go wrong:**

1. **Memory leak** - If you create a reference cycle at the Rc level (counts never reach 0)
2. **Runtime panic** - If you violate borrow rules at the RefCell level (multiple mutable borrows)

### Two Patterns: Which Order?

You'll see two different patterns in this chapter - the order matters!

**Pattern 1: `Rc<RefCell<T>>` - Multiple owners of mutable data**

Use when: You need **multiple owners** who all mutate **the same value**.

```rust
let counter = Rc::new(RefCell::new(0));
let c1 = Rc::clone(&counter);
let c2 = Rc::clone(&counter);

*c1.borrow_mut() += 1;  // Both owners mutate the same counter
*c2.borrow_mut() += 1;  // Counter is now 2
```

**Pattern 2: `RefCell<Rc<T>>` - Changing which shared value you point to**

Use when: You have **one owner** who needs to **change which** `Rc` it points to.

```rust
struct Node {
    next: RefCell<Option<Rc<Node>>>,  // Can change which node this points to
}

let node_a = Rc::new(Node { next: RefCell::new(None) });
let node_b = Rc::new(Node { next: RefCell::new(None) });

// node_a changes which node it points to
*node_a.next.borrow_mut() = Some(Rc::clone(&node_b));
```

**Key difference:**

- `Rc<RefCell<T>>`: Multiple owners → mutate the value inside
- `RefCell<Rc<T>>`: One owner → change which Rc you're pointing to

You'll see **both** patterns in real code!

> **Note:** When you add `Option` into the mix, the order becomes even more important: `RefCell<Option<Rc<T>>>` vs `Option<RefCell<Rc<T>>>` vs `Rc<RefCell<Option<T>>>` all have different capabilities. See [Appendix: Nested Types](appendix-nested-types.md) for a detailed exploration of these patterns.

## Creating Cycles: The Memory Leak Problem

Now that we can mutate through `Rc`, we can create **actual reference cycles** that leak memory. Chapter 7 showed this conceptually with `std::mem::forget` - now let's see it for real.

### Step-by-Step Cycle Creation

```rust
use std::rc::Rc;
use std::cell::RefCell;

// Node that can point to another node
struct Node {
    value: i32,
    next: RefCell<Option<Rc<Node>>>,
}

// Create two nodes
let node_a = Rc::new(Node {
    value: 1,
    next: RefCell::new(None),
});

let node_b = Rc::new(Node {
    value: 2,
    next: RefCell::new(None),
});

// a count: 1, b count: 1

// Point a -> b
*node_a.next.borrow_mut() = Some(Rc::clone(&node_b));
// a count: 1, b count: 2 (a's next points to b)

// Point b -> a (creating a cycle!)
*node_b.next.borrow_mut() = Some(Rc::clone(&node_a));
// a count: 2, b count: 2 (cycle created!)
```

**Visual representation of the cycle:**

```
Stack                    Heap

                       ┌──────────────────────────┐
┌─────────┐            │ Rc<Node> for node_a      │
│ node_a  │───────────>│ strong_count: 2          │
└─────────┘            │ value: 1                 │
                       │ next: Some(Rc) ───────┐  │
                       └───────────────────────┼──┘
                                             ▲ │
                                             │ │
                                             │ ▼
                       ┌─────────────────────┼────┐
┌─────────┐            │ Rc<Node> for node_b │    │
│ node_b  │───────────>│ strong_count: 2     │    │
└─────────┘            │ value: 2            │    │
                       │ next: Some(Rc) ─────┘    │
                       └──────────────────────────┘

                         Circular reference!
```

**When the stack variables drop:**

1. `node_a` drops: Decrements node_a's count from 2 to 1
2. `node_b` drops: Decrements node_b's count from 2 to 1
3. Both nodes still have count = 1 (from each other's `next` field)
4. Neither can be freed because their counts never reach 0
5. Memory leaked forever! 💀

**This is EXACTLY what Chapter 7's `std::mem::forget` example demonstrated** - the memory stays allocated because the reference count never reaches zero, preventing `Drop` from being called.

### Why Cycles Leak Memory

The cycle prevents deallocation because:

- **Node A** can't be freed while Node B holds a reference to it
- **Node B** can't be freed while Node A holds a reference to it
- There's no "starting point" to begin cleanup
- The reference counts stay positive forever

This is a **memory leak** - the memory is allocated but never freed. Unlike other memory safety violations (use-after-free, double-free), memory leaks are considered "safe" in Rust - they won't crash your program, but they waste memory.

> **Note:** "Forever" means for the program's lifetime - the OS reclaims memory when the program exits. The real problem: if cycles are created repeatedly in long-running programs (servers that run 24/7, desktop apps), memory usage grows unbounded until the program crashes.

## Breaking Cycles with Weak

Remember our cycle problem? Both nodes point to each other:

```
node_a (count: 2) ──→ node_b (count: 2)
    ↑                      │
    └──────────────────────┘
```

When we drop both variables, the counts go from 2 to 1, but never reach 0. **The nodes keep each other alive forever.**

### The Real-World Problem: Parent-Child Trees

Imagine a tree structure where:

- Parent nodes need to access their children
- Child nodes need to access their parent

```rust
struct Node {
    value: i32,
    parent: Rc<RefCell<Node>>,   // Child points to parent
    children: Vec<Rc<RefCell<Node>>>,  // Parent points to children
}
```

This creates a cycle! The parent keeps children alive, and children keep the parent alive. When you drop the root, nothing is freed.

### The Solution: Weak References

**The insight:** Children shouldn't "own" their parent. They just need to reference it.

**What does "own" mean?** In Rust, owning a value means **keeping it alive**. When you have an `Rc<T>`, you own the value - it won't be freed as long as your `Rc` exists. The value is only freed when _all_ owners (all `Rc` clones) are dropped.

In a tree, if children own their parent via `Rc`, they keep the parent alive. But the parent also owns the children via `Rc`, keeping them alive. This is a cycle - nothing can be freed.

The fix: Use `Weak<T>` for child → parent references. `Weak` **doesn't own** - it doesn't keep the value alive:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    value: i32,
    parent: Weak<RefCell<Node>>,        // Weak: doesn't own parent
    children: Vec<Rc<RefCell<Node>>>,   // Rc: owns children
}

// Create root node (the parent)
let root = Rc::new(RefCell::new(Node {
    value: 1,
    parent: Weak::new(),  // Root has no parent
    children: vec![],
}));

// Create child nodes
let child1 = Rc::new(RefCell::new(Node {
    value: 2,
    parent: Rc::downgrade(&root),  // Weak reference to parent
    children: vec![],
}));

let child2 = Rc::new(RefCell::new(Node {
    value: 3,
    parent: Rc::downgrade(&root),  // Weak reference to parent
    children: vec![],
}));

// Root stores strong references to children
root.borrow_mut().children.push(Rc::clone(&child1));
root.borrow_mut().children.push(Rc::clone(&child2));

// Reference counts:
// - root: strong=1 (only `root` variable owns it; children have Weak which don't increment strong_count)
// - child1: strong=2 (owned by `child1` variable + root.children)
// - child2: strong=2 (owned by `child2` variable + root.children)
```

Now when you drop the variables:

```rust
drop(child1);  // child1 strong_count: 2 → 1 (still owned by root.children)
drop(child2);  // child2 strong_count: 2 → 1 (still owned by root.children)

drop(root);    // ⬅ This triggers the cleanup cascade:
               // 1. root strong_count: 1 → 0 (no more owners)
               // 2. root is freed
               // 3. root.children Vec is freed
               // 4. child1 strong_count: 1 → 0 (Vec was the last owner)
               // 5. child2 strong_count: 1 → 0 (Vec was the last owner)
               // 6. child1 and child2 are freed
               // 7. No leak! Everything is cleaned up.
```

**Why this works:** Children use `Weak` for parent references, so they don't keep the root alive. When the `root` variable is dropped, the root's strong count reaches 0 and it's freed. This frees the `children` Vec, which drops the children.

### How Weak Works

```rust
let strong = Rc::new(42);
let weak: Weak<i32> = Rc::downgrade(&strong);
// Strong count: 1, Weak count: 1

// Must upgrade to use (might return None if value was dropped)
if let Some(upgraded) = weak.upgrade() {
    // upgrade() succeeded! strong_count: 1 → 2
    // `upgraded` is now an Rc<i32> - a new owner
    assert_eq!(*upgraded, 42);  // Value: 42

    // When `upgraded` goes out of scope: strong_count: 2 → 1
}
// After scope ends, strong_count back to 1

drop(strong); // strong_count: 1 → 0, value is freed (even though weak_count = 1, Weak doesn't keep value alive)

// After value is dropped, upgrade returns None
assert!(weak.upgrade().is_none());
```

**Key differences:**

| Strong (`Rc<T>`)           | Weak (`Weak<T>`)                                  |
| -------------------------- | ------------------------------------------------- |
| Keeps value alive          | Doesn't keep value alive                          |
| Increments `strong_count`  | Increments `weak_count`                           |
| Direct access with `*`     | Must `.upgrade()` first (returns `Option<Rc<T>>`) |
| Value freed when count = 0 | Can access if strong refs still exist             |

### Parent-Child Tree Pattern

The classic use case: Trees where children can navigate to their parent.

**The rule:** Use `Rc` for ownership direction, `Weak` for back-references:

```rust
use std::rc::{Rc, Weak};
use std::cell::RefCell;

struct Node {
    value: i32,
    parent: RefCell<Weak<Node>>,           // ← Weak to parent (non-owning)
    children: RefCell<Vec<Rc<Node>>>,      // ← Strong to children (owning)
}

impl Node {
    fn new(value: i32) -> Rc<Node> {
        Rc::new(Node {
            value,
            parent: RefCell::new(Weak::new()),
            children: RefCell::new(vec![]),
        })
    }

    fn add_child(parent: &Rc<Node>, child: &Rc<Node>) {
        // Child holds Weak reference to parent (doesn't keep parent alive)
        *child.parent.borrow_mut() = Rc::downgrade(parent);

        // Parent holds strong reference to child (keeps child alive)
        parent.children.borrow_mut().push(Rc::clone(child));
    }
}

// Usage
let root = Node::new(1);
let child1 = Node::new(2);
let child2 = Node::new(3);

Node::add_child(&root, &child1);
Node::add_child(&root, &child2);

// Child can access parent
if let Some(parent) = child1.parent.borrow().upgrade() {
    println!("Parent value: {}", parent.value); // 1
}

// When root is dropped:
// - Children's strong_count goes to 0 (only parent held them)
// - Children are freed
// - Parent's weak_count goes to 0 (children held weak refs)
// - No memory leak! ✅
```

**Visual representation:**

```
┌─────────────┐
│    root     │<─.──.──.──.──.──.──.──.──.──.──.──.──.──.
│   value: 1  │                                         │
└─────┬───────┘                                         .
      │                                                 │
      │ Strong references (parent owns children)        .
      │                                                 │
      ├──────────────┬──────────────┐                   .
      ▼              ▼              ▼                   │
┌──────────┐   ┌──────────┐   ┌──────────┐              .
│  child1  │   │  child2  │   │  child3  │              │
│ value: 2 │   │ value: 3 │   │ value: 4 │              .
└──────────┘   └──────────┘   └──────────┘              │
      │              │              │                   .
      .──.──.──.──.──.──.──.──.──.──.                   │
                     │                                  .
           Weak references (children don't own parent)  │
                     │                                  .
                     .──.──.──.──.──.──.──.──.──.──.──.─┘
```

**Why this works:**

- Parent owns children with `Rc` (strong references)
- Children reference parent with `Weak` (non-owning references)
- When parent drops, children's `strong_count` goes to 0, so they're freed
- No cycle, no leak! ✅

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

## Implementation: Building Weak<T>

> **Note:** The complete implementation is in [src/rc.rs](../../src/rc.rs) in this repository. That implementation already includes `Weak<T>` and uses `ManuallyDrop<T>` internally. These sections show how to build them step-by-step to help you understand how they work under the hood.

### Weak Structure

`Weak<T>` has the same structure as `Rc<T>` - it's just a pointer to the same `RcInner`:

```rust
use std::cell::Cell;

struct RcInner<T> {
    strong_count: Cell<usize>,
    weak_count: Cell<usize>,  // Track weak references
    value: T,
}

pub struct Rc<T> {
    ptr: *mut RcInner<T>,
}

pub struct Weak<T> {
    ptr: *mut RcInner<T>,  // Same pointer type as Rc!
}
```

**Key difference:** `Weak` points to the same allocation as `Rc`, but doesn't keep the value alive.

> **Implementation Note: Implicit Weak Reference**
> In the real `std::rc::Rc` implementation, `weak_count` starts at 1, not 0. This represents an "implicit weak reference" held by the `Rc` allocation itself. The count only reaches 0 when both:
>
> 1. All strong references are dropped (`strong_count == 0`)
> 2. All explicit weak references are dropped
>
> This pattern ensures the `RcInner` allocation stays valid as long as ANY reference (strong or weak) exists. When `strong_count` reaches 0, the value `T` is dropped but `RcInner` remains allocated with `weak_count = 1` (the implicit reference). Only when the last explicit `Weak` is dropped does `weak_count` go from 1 to 0, triggering deallocation of `RcInner`.
>
> For simplicity, our implementation here shows `weak_count` starting at 0, which is conceptually easier to understand but differs from the actual std library implementation.

### Creating Weak: Rc::downgrade()

Convert a strong reference to a weak one:

```rust
impl<T> Rc<T> {
    pub fn downgrade(this: &Rc<T>) -> Weak<T> {
        unsafe {
            let inner = &*this.ptr;

            // Increment weak_count
            let count = inner.weak_count.get();
            inner.weak_count.set(count + 1);

            // Create Weak pointing to the same allocation
            Weak { ptr: this.ptr }
        }
    }
}
```

**What happens:**

1. Increment `weak_count` (doesn't affect `strong_count`)
2. Create a new `Weak` with the same pointer
3. Value stays alive (strong_count unchanged)

### Upgrading Weak: Weak::upgrade()

Try to convert a weak reference back to a strong one:

```rust
impl<T> Weak<T> {
    pub fn upgrade(&self) -> Option<Rc<T>> {
        // Check for null pointer (from Weak::new())
        if self.ptr.is_null() {
            return None;
        }

        unsafe {
            let inner = &*self.ptr;
            let strong = inner.strong_count.get();

            // Check if value is still alive
            if strong == 0 {
                // Value has been dropped, can't upgrade
                None
            } else {
                // Value still alive, increment strong_count
                inner.strong_count.set(strong + 1);

                // Create new Rc pointing to same allocation
                Some(Rc { ptr: self.ptr })
            }
        }
    }
}
```

**Why it returns `Option`:**

- If `strong_count == 0`, all `Rc` owners have dropped, value is gone → return `None`
- If `strong_count > 0`, value still alive → increment count and return `Some(Rc)`

**Example:**

```rust
let strong = Rc::new(42);
let weak = Rc::downgrade(&strong);
// strong_count = 1, weak_count = 1

// Upgrade succeeds (strong_count > 0)
if let Some(rc) = weak.upgrade() {
    assert_eq!(*rc, 42);
    // strong_count now 2
}
// strong_count back to 1

drop(strong);
// strong_count = 0, value dropped

// Upgrade fails (strong_count == 0)
assert!(weak.upgrade().is_none());
```

### Weak::new() - Creating an Empty Weak

```rust
impl<T> Weak<T> {
    pub fn new() -> Weak<T> {
        Weak {
            ptr: std::ptr::null_mut(),  // Null pointer, doesn't point to anything
        }
    }
}
```

**Usage:** For fields that might never have a value:

```rust
struct Node {
    parent: Weak<Node>,  // Start with no parent
}

let root = Node {
    parent: Weak::new(),  // Root has no parent
};
```

### Weak's Drop Implementation

When a `Weak` is dropped, we decrement `weak_count` and potentially deallocate:

```rust
impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        // Check for null pointer (from Weak::new())
        if self.ptr.is_null() {
            return;
        }

        unsafe {
            let inner = &*self.ptr;
            let weak = inner.weak_count.get();
            inner.weak_count.set(weak - 1);

            if weak - 1 == 0 {
                // Last Weak reference!

                let strong = inner.strong_count.get();
                if strong == 0 {
                    // Value already dropped by Rc, safe to deallocate
                    drop(Box::from_raw(self.ptr));
                }
                // If strong > 0, some Rc still exists, they'll handle deallocation
            }
        }
    }
}
```

**The lifecycle:**

```rust
let strong1 = Rc::new(String::from("data"));
let strong2 = Rc::clone(&strong1);
let weak = Rc::downgrade(&strong1);
// strong_count = 2, weak_count = 1

drop(strong1);
// strong_count: 2 → 1
// Nothing deallocated

drop(strong2);
// strong_count: 1 → 0
// Rc::drop() drops the String
// But weak_count = 1, so RcInner stays allocated

drop(weak);
// weak_count: 1 → 0
// strong_count already 0, so Weak::drop() deallocates RcInner
```

**Key insight:** `Weak::drop()` only deallocates `RcInner` when BOTH counts are 0. It never drops the value - `Rc::drop()` handles that.

### Why Weak Doesn't Keep Value Alive

The key is in `upgrade()`:

```rust
pub fn upgrade(&self) -> Option<Rc<T>> {
    if strong_count == 0 {
        return None;  // Value gone, can't access it
    }
    // ...
}
```

**Contrast with `Rc::clone()`:**

```rust
impl<T> Clone for Rc<T> {
    fn clone(&self) -> Rc<T> {
        // Always increments strong_count
        let count = self.inner().strong_count.get();
        self.inner().strong_count.set(count + 1);
        Rc { ptr: self.ptr }
    }
}
```

**The difference:**

- `Rc::clone()` → increments `strong_count` → keeps value alive
- `Rc::downgrade()` → increments `weak_count` → doesn't keep value alive
- `Weak::upgrade()` → checks `strong_count` first → returns `None` if value dropped

### Complete Weak Example

```rust
use std::rc::{Rc, Weak};

// Create strong reference
let strong = Rc::new(42);
println!("strong_count: {}", Rc::strong_count(&strong)); // 1

// Create weak reference
let weak: Weak<i32> = Rc::downgrade(&strong);
println!("weak_count: {}", Rc::weak_count(&strong)); // 1

// Upgrade weak to strong (value still alive)
if let Some(upgraded) = weak.upgrade() {
    println!("Upgraded: {}", *upgraded); // 42
    println!("strong_count: {}", Rc::strong_count(&upgraded)); // 2
}
// upgraded dropped, strong_count back to 1

// Drop the strong reference
drop(strong);

// Try to upgrade (value is gone)
match weak.upgrade() {
    Some(_) => println!("Upgraded successfully"),
    None => println!("Can't upgrade, value dropped"), // This runs
}
```

## Implementation: Why Rc Needs ManuallyDrop

### Quick Reminder: Rc Structure

From Chapter 7, recall that `Rc<T>` is defined as:

```rust
use std::cell::Cell;

struct RcInner<T> {
    strong_count: Cell<usize>,  // Reference count
    value: T,                   // The actual data
}

pub struct Rc<T> {
    ptr: *mut RcInner<T>,  // Pointer to heap allocation
}
```

When implementing `Drop` for `Rc<T>`, we face a **double-drop** problem. This affects `Rc<T>` for **any** type `T`, not just `RefCell`.

### The Problem: Double Drop

**Concrete example:** Let's say we have 2 strong references and 1 weak reference:

```rust
use std::rc::{Rc, Weak};

let strong1 = Rc::new(String::from("data"));
let strong2 = Rc::clone(&strong1);
let weak: Weak<String> = Rc::downgrade(&strong1);

// State: strong_count = 2, weak_count = 1
```

**What needs to happen when we drop both strong refs:**

1. Drop `strong1` → `strong_count` = 2 → 1 (value still alive)
2. Drop `strong2` → `strong_count` = 1 → 0 (value must be dropped now!)
3. But `weak` still exists! `RcInner` must stay allocated so `weak.upgrade()` can check `strong_count`
4. Later, drop `weak` → `weak_count` = 1 → 0 (now deallocate `RcInner`)

**The challenge:** In step 2, we must **drop the String** but **keep the RcInner allocation alive**. These are two separate operations.

### Naive Approach (Broken)

Now let's see the implementation problem:

```rust
impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        unsafe {
            // self.ptr is *mut RcInner<T>
            let inner = &*self.ptr;
            let count = inner.strong_count.get();
            inner.strong_count.set(count - 1);

            if count - 1 == 0 {
                // Last strong owner! (strong_count = 0, but weak_count = 1)

                // Step 1: Drop the value T
                ptr::drop_in_place(&mut (*self.ptr).value);
                // ✅ String is now dropped, its buffer freed

                // Step 2: Check if we can deallocate RcInner
                let weak = inner.weak_count.get();
                if weak == 0 {
                    // No Weak references, safe to deallocate
                    drop(Box::from_raw(self.ptr));
                    // ⚠️ PROBLEM: Box::drop will try to drop all fields of RcInner<T>
                    // This includes `value: T` - but we already dropped it in Step 1!
                    // 💥 DOUBLE DROP!
                } else {
                    // weak_count = 1, Weak reference still exists!
                    // We must keep the RcInner allocation alive so Weak::upgrade() can check strong_count
                    // The String is dropped, but RcInner stays allocated
                    // Later when Weak is dropped, it will deallocate RcInner
                }
            }
        }
    }
}
```

**Why we need TWO separate steps:**

When `strong_count` reaches 0:

1. **Drop the value** - No more strong owners, value must be freed
2. **Keep the RcInner allocation** - If `weak_count > 0`, Weak references still need to check `strong_count`

When `weak_count` ALSO reaches 0: 3. **Deallocate the RcInner** - No more references at all

**The double-drop problem:**

If we skip Step 1 and just do Step 3 (deallocate RcInner), we can't keep the allocation alive for Weak references. We need to drop the value when strong_count → 0, but deallocate the struct only when weak_count → 0.

Without `ManuallyDrop`, step 3 would automatically drop `value` again - but we already dropped it in step 1!

### The Solution: ManuallyDrop

`ManuallyDrop<T>` is a wrapper that tells Rust: "Don't automatically drop this value - I'll handle it manually."

**How does it do this?** Simple: by not implementing `Drop`. When Rust drops a struct, it automatically drops all its fields - but only if the field's type implements `Drop`. Since `ManuallyDrop<T>` doesn't implement `Drop`, Rust won't automatically drop the inner `T`.

```rust
use std::mem::ManuallyDrop;
use std::cell::Cell;

struct RcInner<T> {
    strong_count: Cell<usize>,
    weak_count: Cell<usize>,
    value: ManuallyDrop<T>,  // ← Wrapped in ManuallyDrop
}
```

Now with `ManuallyDrop`, the Drop implementation is safe:

```rust
impl<T> Drop for Rc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = &*self.ptr;
            let count = inner.strong_count.get();
            inner.strong_count.set(count - 1);

            if count - 1 == 0 {
                // Manually drop the value (need mutable access)
                let inner_mut = &mut *self.ptr;
                ManuallyDrop::drop(&mut inner_mut.value);  // ✅ Drops T inside ManuallyDrop

                let weak = inner.weak_count.get();
                if weak == 0 {
                    // Deallocate the RcInner
                    // Since value is ManuallyDrop, Rust won't try to drop it again
                    drop(Box::from_raw(self.ptr));  // ✅ Safe! No double-drop
                }
            }
        }
    }
}
```

**Why it works:**

1. `ManuallyDrop<T>` prevents automatic drop when `RcInner` is deallocated
2. We explicitly call `ptr::drop_in_place()` on the value when `strong_count` reaches 0
3. No double drop - the value is dropped exactly once

### What is ManuallyDrop?

`ManuallyDrop<T>` is a zero-cost wrapper defined in `std::mem`:

```rust
#[repr(transparent)]
pub struct ManuallyDrop<T> {
    value: T,
}
```

**Key properties:**

- **Zero cost**: Same size and layout as `T` (due to `#[repr(transparent)]`)
- **Does not implement Drop**: Rust never automatically drops the inner `T`
- **Requires unsafe to drop**: You must explicitly call `ManuallyDrop::drop(&mut x)`

**Core operations:**

```rust
// Create
let x = ManuallyDrop::new(String::from("hello"));

// Access (no drop)
println!("{}", *x);  // Deref to access inner value

// Drop explicitly (unsafe - you must ensure no future access)
unsafe {
    ManuallyDrop::drop(&mut x);  // Drops the String
}
// After this, accessing x is undefined behavior!

// Move out without dropping (unsafe)
let s = unsafe { ManuallyDrop::take(&mut x) };  // Takes ownership of String
// Now s owns the String, x is left in uninitialized state
```

### Can We Manually Implement It?

Yes! Here's how `ManuallyDrop` is essentially implemented:

```rust
use std::mem::forget;
use std::ptr;

#[repr(transparent)]
pub struct ManualDrop<T> {
    value: T,
}

impl<T> ManualDrop<T> {
    pub const fn new(value: T) -> Self {
        ManualDrop { value }
    }

    // Safe: Just creates a reference
    pub fn deref(&self) -> &T {
        &self.value
    }

    // Unsafe: Caller must ensure no further access
    pub unsafe fn drop(slot: &mut Self) {
        // Read the value out and drop it
        ptr::drop_in_place(&mut slot.value);
    }

    // Unsafe: Caller must ensure no further access
    pub unsafe fn take(slot: &mut Self) -> T {
        // Move the value out, leaving uninitialized memory
        ptr::read(&slot.value)
    }
}

// No Drop implementation! This is the key.
// Without Drop, Rust won't automatically drop T.

impl<T> Deref for ManualDrop<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}
```

**Key insight:** By _not_ implementing `Drop`, the wrapper doesn't drop its contents. You must manually call `drop()` or `take()` to handle the inner value.

### ManuallyDrop in Practice

You'll see `ManuallyDrop` used whenever:

1. **Reference counting** - `Rc`, `Arc` need to control when `T` is dropped
2. **Custom allocators** - Managing memory lifetime manually
3. **FFI boundaries** - Passing ownership to C code
4. **Union types** - Only one variant should be dropped
5. **Implementing Drop flags** - Control drop order precisely

**Warning:** Using `ManuallyDrop` is subtle - you must ensure:

- Drop is called exactly once
- No access after drop
- Memory is eventually freed

Get it wrong, and you'll have memory leaks or use-after-free bugs!

## Anti-patterns

```rust
// ❌ BAD: Single owner, no sharing needed
let data = Rc::new(RefCell::new(vec![1, 2, 3]));
data.borrow_mut().push(4);
// Should be: let mut data = vec![1, 2, 3];

// ❌ BAD: Never mutated, just shared
let config = Rc::new(RefCell::new(Config::load()));
// Should be: let config = Rc::new(Config::load());

// ❌ BAD: Using in multi-threaded context
let counter = Rc::new(RefCell::new(0));
thread::spawn(move || { ... });  // Won't compile! (Not Send)
// Should be: Arc<Mutex<i32>> or Arc<AtomicI32>
```

## Key Takeaways

1. **Rc<RefCell<T>> combines shared ownership + interior mutability** - Multiple owners can all mutate the same data
1. **Two levels of tracking** - Rc tracks reference counts (ownership), RefCell tracks borrows (usage)
1. **Two ways to fail** - Memory leaks from Rc cycles (safe but wasteful), panics from RefCell borrow violations (runtime error)
1. **Use Weak to break cycles** - Weak references don't keep values alive, preventing memory leaks in cyclic structures
1. **Parent-child pattern** - Use Rc for ownership direction (parent → child), Weak for back-references (child → parent)
1. **Drop borrows quickly** - Keep RefCell borrows short-lived to avoid runtime panics
1. **Don't overuse** - If you have a clear owner or don't need sharing, use simpler patterns (&mut T, Box<T>, Rc<T>)
1. **Always check Weak::upgrade()** - Weak references might point to dropped values, always handle None case
1. **Runtime cost** - Both Rc (reference counting) and RefCell (borrow checking) have runtime overhead, use only when needed

## Exercises

See [examples/08_rc_refcell.rs](../examples/08_rc_refcell.rs) for hands-on exercises demonstrating:

- Creating and using `Rc<RefCell<T>>`
- Multiple owners mutating shared data
- Creating and breaking reference cycles with `Weak`
- Building graphs and trees with cycles
- Observer pattern implementation
- Common pitfalls and how to avoid them

**Complete solutions:** Switch to the `answers` branch with `git checkout answers` to see completed exercises.

## Next Chapter

[Chapter 9: Arc - Atomic Reference Counting](./09-arc.md) - Thread-safe shared ownership for multi-threaded code.
