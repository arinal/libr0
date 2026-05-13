# Chapter 7: Rc - Reference Counting

## The Real Problem: Multiple Owners Need the Same Data

Rust's ownership model says every value has exactly one owner. This prevents many bugs, but sometimes multiple parts of your code legitimately need to own the same data:

```rust
struct Config {
    db_url: String,
    api_key: String,
}

// This won't work - who owns the config?
let config = Config::load();
let server = Server::new(config);    // server takes ownership
let logger = Logger::new(config);    // ❌ ERROR: config already moved!
```

**You can't**:

- Clone `Config` each time (expensive, inconsistent if mutated)
- Use references (lifetime gets complicated with multiple owners)
- Use `Box` (still single ownership)

**You need**: Multiple owners to share the same data.

## What Rc Actually Does

`Rc<T>` (Reference Counted) enables **shared ownership** by tracking how many owners exist:

```rust
use std::rc::Rc;

let config = Rc::new(Config::load());
let server = Server::new(Rc::clone(&config));  // ✅ +1 owner
let logger = Logger::new(Rc::clone(&config));  // ✅ +1 owner
// config, server, and logger all share ownership of the same Config
```

**How it works**:

1. Allocates the value on the heap (like `Box`)
2. Keeps a **reference count** alongside the value
3. Each `Rc::clone()` increments the count (and creates a new pointer)
4. Each `drop` decrements the count
5. When count reaches 0, the value is freed

## Why Rc? Comparing the Three Approaches

### The Key Insight: Rc = Multiple Pointers to THE SAME Heap Data

**With `Rc<T>` (What we want):**

```bob
     STACK            |               HEAP
                      |
+----------+          |        +---------------------+
| Rc       |          |        | Config              |
| ptr *----+------+---+------->+---------------------+
+----------+      |   |        | db_url: String      |     
                  |   |        | api_key: String     |     
                  |   |        +--------^------------+
+----------+      |   |                 |
| Rc       |      |   |                 o
| ptr *----+------+   |        ALL three Rc pointers
+----------+      |   |        point to THIS SAME allocation!
                  |   |                 
+----------+      |   |
| Rc       |      |   |
| ptr *----+------+   |
+----------+          |
```

✅ **Multiple owners, one allocation, shared data**

- All three `Rc` point to the **exact same** `Config` in memory
- Memory is freed only when the last owner drops
- For mutation, see Chapter 13 (Rc + RefCell)

### Why Not `Box<T>`?

**With `Box<T>` (Doesn't work for sharing):**

```bob
     STACK            |               HEAP
                      |
+----------+          |        +-------------+
| Box ptr  |----------|------->| Config copy |  <- Separate allocation
+----------+          |        +-------------+
                      |
+----------+          |        +-------------+
| Box ptr  |----------|------->| Config copy |  <- Another separate allocation
+----------+          |        +-------------+
                      |
+----------+          |        +-------------+
| Box ptr  |----------|------->| Config copy |  <- Yet another allocation
+----------+          |        +-------------+
```

❌ **Problem: Each Box owns a DIFFERENT copy**

- You'd have to clone the `Config` for each component (expensive!)
- Each copy is independent - changes to one don't affect others
- Three separate heap allocations wasting memory
- `Box::new(config)` **moves** `config`, making it unavailable for the next component

```rust
let config = Config::load();
let server = Box::new(config);  // config moved here
let logger = Box::new(config);  // ❌ ERROR: config already moved!
```

### Why Not References `&T`?

**With references (Lifetime hell):**

```bob
   Stack                 Heap "(or Stack)"

+----------+            +-------------+
| &config  |----------->|   Config    |  <- All references point to
+----------+            +-------------+     the same data "(good!)"
                               ^
+----------+                   |
| &config  |-------------------+            But WHO owns it?
+----------+                   |
                               |
+----------+                   |
| &config  |-------------------+
+----------+
```

❌ **Problem: References don't own - lifetimes get complex**

First, let's define structs that use references:

```rust
struct Server<'a> { config: &'a Config }
struct Logger<'a> { config: &'a Config }
```

### Problem 1: Can't return from functions

When you try to return a struct containing a reference to local data, it fails:

```rust
fn create_server() -> Server {
    let config = Config::load();
    let server = Server { config: &config };
    server  // ❌ ERROR: cannot return value referencing local variable `config`
}
```

**Why it fails:** `config` is dropped at the end of the function, but `server.config` still references it!

### Problem 2: Lifetimes infect everything like a virus

You **can** store references locally - that works fine:

```rust
let config = Config::load();
let server = Server { config: &config };
let logger = Logger { config: &config };

// Storing in a Vec works locally
let mut components: Vec<Server<'_>> = Vec::new();
components.push(server);  // ✅ This works here!
```

**But now the lifetime constraint INFECTS everything that touches it:**

Want to return the `Vec`? Now the function needs lifetimes:

```rust
fn make_components<'a>(cfg: &'a Config) -> Vec<Server<'a>> {
    vec![Server { config: cfg }]
}
```

Want to store in a struct? Now the struct needs lifetimes:

```rust
struct App<'a> {
    components: Vec<Server<'a>>,
}
```

Want to store `App` in another struct? That needs lifetimes too:

```rust
struct System<'a> {
    app: App<'a>,
}
```

Every function that uses `System` needs lifetimes:

```rust
fn process_system<'a>(sys: &System<'a>) {
    // Process the system
}
```

**This cascades through your ENTIRE codebase - all because `Server` contains a reference!**

## How Rc Solves These Problems

Now let's see how `Rc` fixes both issues:

### Solution to Problem 1: Returning from functions works

With `Rc`, you can return structs containing shared data - no lifetime issues:

```rust
use std::rc::Rc;

struct Server { config: Rc<Config> }  // No lifetime parameter!

fn create_server() -> Server {
    let config = Rc::new(Config::load());
    Server { config }  // ✅ Works! Rc owns the data
}
```

**Why it works:** `Rc` **owns** the data (keeps it alive), unlike references which just borrow. When you return `Server`, you're moving ownership of the `Rc` out of the function.

### Solution to Problem 2: No lifetime infection

With `Rc`, no lifetime parameters needed anywhere:

```rust
use std::rc::Rc;

// No lifetime parameters!
struct Server { config: Rc<Config> }
struct Logger { config: Rc<Config> }

// Storing in a Vec - no lifetime needed
let config = Rc::new(Config::load());
let server = Server { config: Rc::clone(&config) };
let logger = Logger { config: Rc::clone(&config) };

let mut components = Vec::new();
components.push(server);  // ✅ Works!

// Functions don't need lifetime parameters
fn make_components(cfg: Rc<Config>) -> Vec<Server> {
    vec![Server { config: cfg }]
}

// Structs don't need lifetime parameters
struct App {
    components: Vec<Server>,
}

struct System {
    app: App,
}

// Functions using System don't need lifetime parameters
fn process_system(sys: &System) {
    // Process the system
}
```

**Why it works:** `Rc` provides **ownership**, not just borrowing. Lifetime tracking is moved from compile-time to runtime - instead of the compiler tracking lifetimes with `'a` annotations, `Rc` tracks them at runtime with reference counts.

**The key trade-off:**

- **References (`&T`)**: Compile-time lifetime tracking → zero runtime overhead, but inflexible
- **Rc**: Runtime reference counting → flexible ownership, but pays cost of counter increments/decrements

### The lifetime problem:

- If `Server` and `Logger` store `&Config`, they must live **shorter** than `config`
- You can't return them from functions (references stack-local data)
- You can't store them in collections easily (lifetime annotations everywhere)
- Complex ownership patterns become impossible to express

**When references work well:**

- Temporary borrows (function parameters)
- Short-lived access patterns
- When there's a clear owner and the borrow is brief

**When you need `Rc` instead:**

- No clear single owner
- Multiple components need to outlive each other independently
- Dynamic lifetimes (can't determine at compile time)
- Building complex data structures (graphs, trees with shared nodes)

### Summary: Why Rc Wins

| Approach | Shares Data?   | Multiple Owners?     | Lifetime Issues?              |
| -------- | -------------- | -------------------- | ----------------------------- |
| `Rc<T>`  | ✅ Yes         | ✅ Yes               | ✅ No - runtime counted       |
| `Box<T>` | ❌ No (copies) | ❌ No (single owner) | ✅ No                         |
| `&T`     | ✅ Yes         | ❌ No (borrowed)     | ❌ Yes - compile-time tracked |

**`Rc<T>` gives you the best of both worlds:**

- Shared data like references (all point to same allocation)
- Multiple ownership like separate boxes (but without the copies)
- Simple lifetimes (no `'a` annotations needed)
- Runtime reference counting handles cleanup automatically

