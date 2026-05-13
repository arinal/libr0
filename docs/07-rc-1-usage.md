# Usage, Patterns & Pitfalls

## Using Rc: Basic Examples

### Example 1: Shared Configuration

```rust
use std::rc::Rc;

let config = Rc::new(Config {
    db_url: String::from("localhost:5432"),
    api_key: String::from("secret"),
});

println!("Strong count: {}", Rc::strong_count(&config)); // 1

let server_config = Rc::clone(&config);  // Now 2 owners
let logger_config = Rc::clone(&config);  // Now 3 owners

println!("Strong count: {}", Rc::strong_count(&config)); // 3

// All three can access the same data
println!("Server sees: {}", server_config.db_url);
println!("Logger sees: {}", logger_config.db_url);
```

### Example 2: Deref Coercion Works

Because `Rc` implements `Deref`, you can use it like a reference:

```rust
let name = Rc::new(String::from("Alice"));

// Deref coercion: Rc<String> -> &String -> &str
fn print_name(s: &str) {
    println!("Name: {}", s);
}

print_name(&name);  // ✅ Works! &Rc<String> coerces to &str
println!("Length: {}", name.len());  // ✅ Call String methods directly
```

### Example 3: Automatic Cleanup via RAII

**RAII (Resource Acquisition Is Initialization)**: When a variable goes out of scope, its `Drop` is called automatically.

```rust
{
    let data = Rc::new(vec![1, 2, 3]);
    println!("Count: {}", Rc::strong_count(&data));  // 1

    {
        let shared = Rc::clone(&data);
        println!("Count: {}", Rc::strong_count(&data));  // 2
    } // `shared` dropped here, count decrements to 1

    println!("Count: {}", Rc::strong_count(&data));  // 1
} // `data` dropped here, count goes to 0, memory freed
```

**Why this matters**: You don't need to manually manage the reference count. Rust's ownership system handles it automatically through `Drop`.

## Why Clone is Cheap

### The Confusion: Two Different "Clones"

```rust
let data = vec![1, 2, 3];
let clone1 = data.clone();  // ❌ EXPENSIVE: Copies all elements

let rc_data = Rc::new(vec![1, 2, 3]);
let clone2 = rc_data.clone();  // ✅ CHEAP: Just increments a counter
```

**Visual comparison:**

**Cloning a `Vec` directly - EXPENSIVE (O(n)):**

```bob
    Before clone             |         After clone
                             |
Stack           Heap         |    Stack           Heap
+------+       +---+---+---+ |  +------+       +---+---+---+
| data |------>| 1 | 2 | 3 | |  | data |------>| 1 | 2 | 3 |
+------+       +---+---+---+ |  +------+       +---+---+---+
                             | 
                             |  +--------+     +---+---+---+
                             |  | clone1 |---->| 1 | 2 | 3 |  <- NEW copy!
                             |  +--------+     +---+---+---+
                             |
                             |  Two separate allocations, all elements copied
```

**Cloning an `Rc<Vec>` - CHEAP (O(1)):**

```bob
Before":"                          |       After Rc"::clone:"
                                   |
Stack             Heap             |   Stack             Heap
                                   |
+---------+      +--------------+  |  +---------+     +--------------+
| rc_data |----->| "count:"1    |  |  | rc_data |-+-->| "count:"2    |
+---------+      | +---+---+---+|  |  +---------+ |   | +---+---+---+|
                 | | 1 | 2 | 3 ||  |              |   | | 1 | 2 | 3 ||
                 | +---+---+---+|  |  +--------+  |   | +---+---+---+|
                 +--------------+  |  | clone2 |--+   +--------------+
                                   |  +--------+
                                   
                                 Same allocation, counter incremented
```

**`Rc::clone()` only clones the pointer, not the data!**

- Cloning the `Rc` = increment counter + copy pointer (O(1))
- Cloning the inner data = depends on the data (could be O(n))

### Convention: Use `Rc::clone(&rc)` for Clarity

```rust
let rc = Rc::new(String::from("hello"));

// Both work, but one is clearer:
let clone1 = rc.clone();           // Looks like it might be expensive
let clone2 = Rc::clone(&rc);       // ✅ Clearly cloning the Rc, not the String
```

The explicit `Rc::clone(&rc)` syntax makes it obvious you're doing a cheap pointer clone, not an expensive data clone.

## Rc Only Gives You Shared References

**Important limitation**: `Rc<T>` only provides `&T`, never `&mut T`.

```rust
let data = Rc::new(vec![1, 2, 3]);
let borrowed: &Vec<i32> = &*data;  // ✅ Can get &T
// let mut_borrowed: &mut Vec<i32> = &mut *data;  // ❌ ERROR: cannot borrow as mutable
```

**Why?** If multiple owners could get `&mut T`, you'd have multiple mutable references to the same data - a data race!

**Solution for mutation**: Use `Rc<RefCell<T>>` (covered in Chapter 13) for interior mutability.

## Don't Confuse get_mut with Rc's Purpose

Like `Cell` and `RefCell` (see Chapters 5-6), `Rc` has a `get_mut()` method - but it's **not the main point**:

```rust
let mut rc = Rc::new(5);
if let Some(data) = Rc::get_mut(&mut rc) {
    *data += 1;  // Requires &mut Rc<T> AND strong_count == 1
}
```

**The key distinction** (same as Cell/RefCell):

- **`get_mut()`**: Requires `&mut self` → compile-time checked → defeats the purpose
- **`Rc::clone()`**: Only needs `&self` → shared ownership → **this is the point!**

If you're the sole owner (`strong_count == 1`), you don't need `Rc` at all! The whole point of `Rc` is enabling **multiple** owners.

## Common Patterns

### Pattern 1: Shared Data Across Components

The most common use case - multiple components need read access to the same data:

```rust
struct Server {
    config: Rc<Config>,
}

struct Logger {
    config: Rc<Config>,
}

struct Database {
    config: Rc<Config>,
}

let config = Rc::new(Config::load());
let server = Server { config: Rc::clone(&config) };
let logger = Logger { config: Rc::clone(&config) };
let db = Database { config: Rc::clone(&config) };

// All components can read the same config
println!("Server using: {}", server.config.db_url);
println!("Logger using: {}", logger.config.db_url);
```

### Pattern 2: Tree Structures (Parent → Children)

`Rc` works well for tree structures where parents own children:

```rust
use std::rc::Rc;

struct Node {
    value: i32,
    children: Vec<Rc<Node>>,
}

let child1 = Rc::new(Node { value: 1, children: vec![] });
let child2 = Rc::new(Node { value: 2, children: vec![] });

let parent = Rc::new(Node {
    value: 0,
    children: vec![Rc::clone(&child1), Rc::clone(&child2)],
});

// child1 and child2 can be shared elsewhere too
let another_parent = Rc::new(Node {
    value: 10,
    children: vec![Rc::clone(&child1)],  // Shared child!
});
```

### Pattern 3: Caching/Flyweight Pattern

Share immutable data to save memory:

```rust
use std::collections::HashMap;
use std::rc::Rc;

struct FontCache {
    fonts: HashMap<String, Rc<FontData>>,
}

impl FontCache {
    fn get(&mut self, name: &str) -> Rc<FontData> {
        self.fonts.entry(name.to_string())
            .or_insert_with(|| Rc::new(FontData::load(name)))
            .clone()  // Cheap Rc clone, not data clone
    }
}
```

### Pattern 4: Functional Data Structures

Share structure between versions:

```rust
use std::rc::Rc;

#[derive(Debug)]
enum List<T> {
    Cons(T, Rc<List<T>>),
    Nil,
}

let tail = Rc::new(List::Cons(2, Rc::new(List::Cons(3, Rc::new(List::Nil)))));

// Two lists sharing the same tail
let list1 = List::Cons(1, Rc::clone(&tail));
let list2 = List::Cons(0, Rc::clone(&tail));
// Both lists share the [2, 3] portion in memory
```

## The Problem with Cycles: When Rc Leaks Memory

`Rc` can create **memory leaks** if you create reference cycles. Here's the conceptual problem:

Imagine two nodes referencing each other:

```bob
Stack               Heap

+---+      +--------------------+
| a |----->| Rc<Node>           | 
+---+      | "strong_count:"2   | 
           | "next:" *          +<--.
           +---------|----------+   |
                     |              |
                     |              |
                     v              |
           +--------------------+   +
+---+      | Rc<Node>           |   |
| b |----->| "strong_count:"2   |   |
+---+      | "next:" *----------+---'
           +--------------------+
```

**When the stack variables drop:**

- Each node's count decrements by 1 (from stack variables)
- But each node still has count = 1 (from the other node)
- Neither reaches 0, so neither is freed
- Memory leak! 💀

**Why this happens**: Each `Rc` in the cycle keeps the others alive. There's no "starting point" to begin deallocation.

**The solution**: Use `Weak<T>` references to break cycles. `Weak` is a non-owning reference that doesn't keep the value alive. This pattern is covered in Chapter 13 (Rc + RefCell), where you'll learn how to combine `Rc`, `Weak`, and interior mutability for practical data structures like graphs and trees with bidirectional references.

## Rc vs Box vs References

|                  | `Box<T>`       | `Rc<T>`           | `&T`              |
| ---------------- | -------------- | ----------------- | ----------------- |
| Ownership        | Single owner   | Multiple owners   | Borrowed          |
| Heap allocation  | Yes            | Yes               | No                |
| Clone behavior   | Deep copy      | Increment counter | Just copy pointer |
| Runtime overhead | None           | Counter checks    | None              |
| Mutability       | `&mut` via Box | Need `RefCell`    | Follow borrows    |
| Thread-safe?     | No             | No                | Depends on `T`    |
| Use when         | Single owner   | Shared immutable  | Short-lived       |

## Key Takeaways

1. **Rc enables shared ownership** - Multiple owners can access the same data
2. **Clone is cheap** - Only increments a counter, doesn't copy data (O(1))
3. **Only shared references** - `Rc` gives `&T`, never `&mut T` (prevents data races)
4. **Beware of cycles** - `Rc` cycles cause memory leaks (see Chapter 13 for solutions)
5. **Not thread-safe** - Use `Arc` (Chapter 8) for multi-threaded code
6. **Automatic cleanup** - RAII ensures memory is freed when last `Rc` drops
7. **get_mut confusion** - `get_mut()` requires sole ownership, defeating Rc's purpose

## Pitfalls

### Pitfall 1: Cloning Inner Data Instead of Rc

**BAD:** Dereferencing and cloning the inner data:

```rust
let rc1 = Rc::new(vec![1, 2, 3, 4, 5]);

// ❌ This clones the Vec, not the Rc!
let vec_clone = (*rc1).clone();  // Expensive! Copies all elements
```

**FIX:** Clone the `Rc`, not the inner data:

```rust
let rc1 = Rc::new(vec![1, 2, 3, 4, 5]);

// ✅ This clones the Rc - cheap!
let rc2 = Rc::clone(&rc1);  // Just increments counter

// Both point to the same Vec
assert_eq!(Rc::strong_count(&rc1), 2);
```

### Pitfall 2: Trying to Mutate Through Rc

**BAD:** Attempting to get `&mut` from `Rc`:

```rust
let rc = Rc::new(vec![1, 2, 3]);
// rc.push(4);  // ❌ ERROR: cannot borrow as mutable
```

**Why this fails:** `Rc` only provides shared references (`&T`), never mutable references (`&mut T`). This prevents data races when multiple owners exist.

**FIX 1:** If you're the sole owner, use `get_mut()`:

```rust
let mut rc = Rc::new(vec![1, 2, 3]);
if let Some(vec) = Rc::get_mut(&mut rc) {
    vec.push(4);  // ✅ Works if strong_count == 1
}
```

**FIX 2:** Use `Box` or plain values if you don't need sharing:

```rust
// If you don't need shared ownership, don't use Rc!
let mut vec = vec![1, 2, 3];
vec.push(4);  // ✅ Simplest solution
```

## Exercises

See [examples/07_rc.rs](../examples/07_rc.rs) for hands-on exercises demonstrating:

**Complete solutions:** Switch to the `answers` branch with `git checkout answers` to see completed exercises

For cycle prevention with `Weak` and mutable shared data, see Chapter 13 (Rc + RefCell).
