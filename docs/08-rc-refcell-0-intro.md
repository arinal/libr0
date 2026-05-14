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
*counter += 1;  // ❌ ERROR: cannot borrow as mutable
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

```bob
     STACK                |           HEAP
                          |
+-------------+           |  +-----------------------+
|"counter: Rc"|           |  | RcInner               |
|"ptr:" *-----+------+----+->+-----------------------+
+-------------+      |    |  | "strong_count: 3"  <--+--- Rc tracks # of owners
                     |    |  | "weak_count: 0"       |
+--------------+     |    |  |         +-------------+
|"counter1: Rc"|     |    |  | "value:"|RefCell<i32> |
|"ptr:"*-------+-----+    |  |         | "borrow:"0  |<-- RefCell tracks borrows   
+--------------+     |    |  |         |  "value:"2  |   "(after two increments)"
                     |    |  |         +-------------+ 
+--------------+     |    |  +-----------------------+
|"counter2: Rc |     |    |
|"ptr:"*-------+-----+    |
+--------------+          |
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

```bob
    STACK         |              HEAP
                  |
+------------+    |     +--------------------+
|"node_a: Rc"|----|---->| RcInner<Node>      |<--.
+------------+    |     +--------------------+   |
                  |     | "strong_count:"2   |   |
                  |     | "value:"1          |   |
                  |     | "next: Rc" *       |   |
                  |     +------------|-------+   |
                  |               .--'           |
                  |               |              |
                  |               v              |
+------------+    |     +--------------------+   |
|"node_b: Rc"|----|---->| RcInner<Node>      |   |
+------------+    |     +--------------------+   |
                  |     | "strong_count:"2   |   |
                  |     | "value:"2          |   |
                  |     | "next: Rc" *-------+---'
                  |     +--------------------+
                  |      
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

```bob
 "node_a" ---> "inner node_a" ----> "inner node_b"  <-- "node_b"
               "(count: 2)"         "(count: 2)"
                    ^                     |
                    +─────────────────────+
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

```bob
+-------------+
|    root     |<~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~.
|  "value:"1  |                               :
+------+------+                               :
       |                                      :
       | Strong refs, parent owns children    :
       |                                      :
       +----------+--------------+            :
       |          |              |            :
       v          v              v            :
+-----------+ +-----------+ +-----------+     :
|  child1   | |  child2   | |  child3   |     :
| "value: 2"| | "value: 3"| | "value: 4"|     :
+------:----+ +-----:-----+ +-----:-----+     :
       :            :             :           :
       '~~~~~~~~~~~~+~~~~~~~~~~~~~'           :
                    :                         :
   Weak, children do:not own parent           :
                    :                         :
                    '~~~~~~~~~~~~~~~~~~~~~~~~~'
```

**Why this works:**

- Parent owns children with `Rc` (strong references)
- Children reference parent with `Weak` (non-owning references)
- When parent drops, children's `strong_count` goes to 0, so they're freed
- No cycle, no leak! ✅

