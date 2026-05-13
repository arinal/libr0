# move, Size & Misconceptions

## The `move` Keyword: Forcing Ownership

By default, closures capture variables **by reference** (either `&T` or `&mut T`). The `move` keyword forces the closure to **take ownership** of all captured variables.

### Syntax

```rust
let s = String::from("hello");
let closure = move || {
    println!("{}", s);  // Takes ownership of s
};
```

### What `move` Does

**Without `move`** (default), closures capture by reference:

```rust
let s = String::from("hello");
let closure = || println!("{}", s);  // Borrows s as &String
closure();
println!("{}", s);  // ✅ OK: s is still available
```

**Without `move`**, you can also force the closure to consume a captured variable by moving it inside the closure body:

```rust
let s = String::from("hello");
let closure = || {
    let s1 = s;  // Moves s out of the environment (forces ownership)
    println!("{}", s1);
};
closure();
// closure();  // ❌ ERROR: closure is FnOnce, can't call twice
// println!("{}", s);  // ❌ ERROR: s was moved into closure
```

This makes the closure `FnOnce` because it consumes `s` on the first call - **the closure cannot be called multiple times**.

Here's what Rust generates:

```rust
// Compiler-generated struct:
struct ClosureEnv<'a> {
    s: &'a String,  // Captured by reference (not owned!)
}

// Implements FnOnce only:
impl<'a> FnOnce<()> for ClosureEnv<'a> {
    type Output = ();

    extern "rust-call" fn call_once(self, args: ()) -> Self::Output {
        let s1 = *self.s;  // Moves the String out through the reference
        println!("{}", s1);
    }
}
```

Notice: The struct captures `s` by **reference** (`&'a String`), but the closure body moves the value out, making it `FnOnce`.

#### How the closure is instantiated

When you write the closure, Rust automatically creates an instance of the generated struct and captures the environment:

```rust
// What you write:
let s = String::from("hello");
let closure = || {
    let s1 = s;
    println!("{}", s1);
};

// What Rust generates (conceptual):
let s = String::from("hello");
let closure = ClosureEnv {
    s: &s,  // Captures s by reference
};

// When you call the closure:
closure();
// Rust generates:
FnOnce::call_once(closure, ());  // Moves closure, calls call_once
```

The key steps:
1. **Closure creation**: `ClosureEnv { s: &s }` - the struct is instantiated with a reference to `s`
2. **Closure call**: `FnOnce::call_once(closure, ())` - the entire struct is moved into `call_once`
3. **Inside call_once**: `let s1 = *self.s` - the String is moved out through the reference
4. **After call**: The closure is consumed and cannot be called again

**With `move`**, closures take ownership at closure creation time, but can still be called multiple times:

```rust
let s = String::from("hello");
let closure = move || println!("{}", s);  // Takes ownership of s immediately
closure();
closure();  // ✅ OK: closure is Fn, can call multiple times!
// println!("{}", s);  // ❌ ERROR: s was moved at closure creation (line 2)
```

The closure is still `Fn` (not `FnOnce`) because it only **reads** `s`, it doesn't consume it. The `move` keyword affects **when** ownership is transferred (at creation vs at call time), not whether the closure can be called multiple times.

Here's what Rust generates:

```rust
// Compiler-generated struct:
struct ClosureEnv {
    s: String,  // Owned by the closure (moved at creation)
}

// Implements Fn (and FnMut, and FnOnce):
impl Fn<()> for ClosureEnv {
    extern "rust-call" fn call(&self, args: ()) -> Self::Output {
        println!("{}", self.s);  // Only reads self.s, doesn't consume it
    }
}
impl FnMut<()> for ClosureEnv {...}
impl FnOnce<()> for ClosureEnv {...}
```

Notice: The struct **owns** `s` (`String`, not `&String`), and the closure implements `Fn` because `call` only **reads** `self.s` without consuming it. The closure can be called multiple times.

**Key difference:**
- **Without `move`**: Variable is captured by reference; ownership transfer happens **when the closure runs** (if moved in body)
- **With `move`**: Variable ownership is transferred **when the closure is created**

### When to Use `move`

#### Use Case 1: Returning Closures

Closures that capture references can't outlive those references. Use `move` to transfer ownership:

```rust
fn make_greeter(name: String) -> impl Fn() {
    move || println!("Hello, {}!", name)  // Takes ownership of name
}

let greet = make_greeter(String::from("Alice"));
greet();  // "Hello, Alice!"
```

Without `move`, this would fail:

```rust
fn make_greeter(name: String) -> impl Fn() {
    || println!("Hello, {}!", name)  // ❌ ERROR: borrowed value doesn't live long enough
}
```

#### Use Case 2: Spawning Threads

Threads require `'static` lifetime. Use `move` to transfer ownership to the thread:

```rust
use std::thread;

let data = vec![1, 2, 3];

thread::spawn(move || {
    println!("{:?}", data);  // Takes ownership of data
}).join().unwrap();

println!("{:?}", data);  // ❌ ERROR: data was moved
```

Without `move`:

```rust
let data = vec![1, 2, 3];

thread::spawn(|| {
    println!("{:?}", data);  // ❌ ERROR: closure may outlive current function
}).join().unwrap();
```

#### Use Case 3: Async Functions

Similar to threads, async closures often need `move` to avoid lifetime issues:

```rust
async fn process_data(data: Vec<i32>) {
    tokio::spawn(move || async move {
        // Process data asynchronously
        println!("{:?}", data);
    });
}
```

### `move` with Copy Types

For types that implement `Copy` (like `i32`, `bool`, `char`), `move` copies the value instead of moving it:

```rust
let x = 42;
let closure = move || println!("{}", x);  // Copies x (i32 is Copy)
closure();
println!("{}", x);  // ✅ OK: x is still available (was copied, not moved)
```

### `move` with Non-Copy Types

For non-Copy types (like `String`, `Vec`), `move` transfers ownership:

```rust
let s = String::from("hello");
let closure = move || println!("{}", s);  // Moves s (String is not Copy)
closure();
println!("{}", s);  // ❌ ERROR: s was moved
```

### Selective Moving

You can't selectively choose which variables to move - `move` applies to **all** captured variables:

```rust
let x = String::from("x");
let y = String::from("y");

let closure = move || {
    println!("{}", x);  // Both x and y are moved
    // Even if we don't use y, it's still moved
};

// println!("{}", x);  // ❌ ERROR: moved
// println!("{}", y);  // ❌ ERROR: moved (even though not used in closure)
```

If you need selective ownership, clone the values you want to keep:

```rust
let x = String::from("x");
let y = String::from("y");
let y_clone = y.clone();

let closure = move || {
    println!("{}", x);  // x is moved
    println!("{}", y_clone);  // y_clone is moved
};

println!("{}", y);  // ✅ OK: original y is still available
```

### Common Pattern: Clone Before `move`

A common pattern is to clone before using `move`:

```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);
let data_clone = Arc::clone(&data);

std::thread::spawn(move || {
    println!("{:?}", data_clone);  // Uses clone, not original
}).join().unwrap();

println!("{:?}", data);  // ✅ OK: original Arc still available
```

### Key Takeaways

- **`move` forces ownership transfer** of all captured variables
- **Use `move` when**:
  - Returning closures from functions
  - Spawning threads or async tasks
  - The closure needs to outlive the current scope
- **Copy types** (like `i32`) are copied, not moved
- **Non-Copy types** (like `String`) are moved and become unavailable
- **`move` applies to all captures** - you can't selectively move some variables
- **Clone before `move`** if you need to keep the original value

