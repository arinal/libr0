# Chapter 1: Option - The Simplest Enum

## The Problem: Null References

In many languages, any reference can be `null`:

```java
String name = null;
int length = name.length(); // NullPointerException!
```

Tony Hoare, who invented null references, called it his "billion-dollar mistake." Rust solves this with `Option`.

## Null Across Languages

**Languages with `null`:**

```java
// Java
String name = null;  // Can assign null to any reference
```

```javascript
// JavaScript
let name = null; // null is a primitive value
```

```scala
// Even Scala, which is a functional language, still has null lurking around
var name: String = null  // ✅ Compiles - null is allowed
name.length  // Runtime error if null!

// But idiomatic Scala uses Option
val name: Option[String] = None
```

**Languages without `null`:**

```haskell
-- Haskell - no null!
name :: Maybe String
name = Nothing  -- Uses Maybe instead

-- You CANNOT do this in Haskell:
name = null  -- ERROR: null doesn't exist!
```

```rust
// Rust - no null!
let name: Option<String> = None;

// You CANNOT do this in Rust:
let name = null;  // ERROR: null doesn't exist!
```

**Key insight:** Rust and Haskell don't have `null` at all. Instead, they use type-safe alternatives (`Option` in Rust, `Maybe` in Haskell) that force you to handle the absence of a value explicitly.

In Rust, to represent "no value," we use an enum called `Option`, which we'll implement ourselves as `MyOption`.

## Our Option Type

```rust
enum MyOption<T> {
    Some(T),
    None,
}
```

That's it. Two variants:

- `Some(T)` - contains a value of type `T`
- `None` - represents absence of a value

The compiler forces you to handle both cases. You can't accidentally use a `None` as if it were `Some`.

## Basic Usage

```rust
use MyOption::{Some, None};

fn find_user(id: u32) -> MyOption<String> {
    if id == 1 {
        Some(String::from("Alice"))
    } else {
        None
    }
}

fn main() {
    let user = find_user(1);

    // Must handle both cases
    match user {
        Some(name) => println!("Found: {}", name),
        None => println!("User not found"),
    }
}
```

**Why is this better than null?**

Notice that `find_user` returns `MyOption<String>`, not `String`. This is the key difference:

| With null (Java, etc.)                   | With Option (Rust)                     |
| ---------------------------------------- | -------------------------------------- |
| `String find_user(...)`                  | `MyOption<String> find_user(...)`      |
| Return type lies - might be null         | Return type is honest - might be None  |
| Compiler lets you ignore null            | Compiler **forces** you to handle None |
| Crash at runtime: `NullPointerException` | Error at compile time                  |

```java
// Java: Compiler is happy, but this crashes at runtime
String user = findUser(99);
int len = user.length();  // NullPointerException!
```

```rust
// Rust: Compiler is not happy, `user` is MyOption<String>, not String
let user = find_user(99);
let len = user.len();  // Error: MyOption<String> has no method `len`
```

```rust
// You MUST unwrap it first, which forces you to think about None
let len = match user {
    Some(s) => s.len(),
    None => 0,  // You're forced to decide what happens here
};
```

The compiler is your safety net. It won't let you forget.

## Implementing Option Methods

Let's build the most useful methods step by step.

### is_some and is_none

The simplest methods - just check which variant we have:

```rust
impl<T> MyOption<T> {
    fn is_some(&self) -> bool {
        match self {
            Some(_) => true,
            None => false,
        }
    }

    fn is_none(&self) -> bool {
        !self.is_some()
    }
}
```

**Examples:**

```rust
let x: MyOption<u32> = Some(42);
x.is_some()  // true
x.is_none()  // false

let y: MyOption<u32> = None;
y.is_none()  // true
y.is_some()  // false

// Useful for conditional checks
if x.is_some() {
    println!("x has a value");
}

// Or for early returns
fn process(opt: MyOption<i32>) -> Result<(), String> {
    if opt.is_none() {
        return Err("No value provided".to_string());
    }
    // Continue processing...
    Ok(())
}
```

### unwrap - The Dangerous One

Extract the value, panic if `None`:

```rust
impl<T> MyOption<T> {
    fn unwrap(self) -> T {
        match self {
            Some(val) => val,
            None => panic!("called unwrap on a None value"),
        }
    }
}
```

> **Warning**: Only use `unwrap()` when you're 100% sure it's `Some`, or in examples/tests.

**Examples:**

```rust
let x = Some("value");
x.unwrap()  // "value"

// This will panic! Avoid in production code
let y: MyOption<&str> = None;
// y.unwrap();  // ❌ Panics: "called unwrap on a None value"

// Safe uses of unwrap:
// 1. In tests
#[test]
fn test_parse() {
    let result = parse_config("valid_config.json");
    assert_eq!(result.unwrap().port, 8080);  // OK in tests
}

// 2. When you've already checked
let opt = Some(42);
if opt.is_some() {
    let value = opt.unwrap();  // Safe, but pattern matching is cleaner
}

// 3. When failure is a programming error
let config = load_config().unwrap();  // OK if missing config means broken setup
```

### unwrap_or - Safe Default

Provide a fallback value:

```rust
impl<T> MyOption<T> {
    fn unwrap_or(self, default: T) -> T {
        match self {
            Some(val) => val,
            None => default,
        }
    }
}
```

**Examples:**

```rust
// Basic usage
let x = Some(42);
x.unwrap_or(0)  // 42

let y: MyOption<i32> = None;
y.unwrap_or(0)  // 0

// User input with fallback
fn get_count(user_input: MyOption<i32>) -> i32 {
    user_input.unwrap_or(10)  // Default to 10 if no input
}

get_count(Some(5))  // 5
get_count(None)     // 10
```

### unwrap_or_else - Lazy Default

Sometimes computing the default is expensive. Only compute it if needed:

```rust
impl<T> MyOption<T> {
    fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Some(val) => val,
            None => f(),
        }
    }
}
```

**Examples:**

```rust
// Basic usage
let x = Some(42);
x.unwrap_or_else(|| 0)  // 42

let y: MyOption<i32> = None;
y.unwrap_or_else(|| 0)  // 0

// Avoid expensive computation when Some
fn expensive_computation() -> String {
    println!("Computing...");  // This won't print if Some
    String::from("default")
}

let some_value = Some(String::from("existing"));
let result = some_value.unwrap_or_else(|| expensive_computation());
// "Computing..." is NOT printed because we have Some
result  // "existing"

let none_value: MyOption<String> = None;
let result = none_value.unwrap_or_else(|| expensive_computation());
// "Computing..." IS printed because we have None
result  // "default"

// Database lookup as fallback
fn find_in_cache(key: &str) -> MyOption<String> { None }
fn fetch_from_db(key: &str) -> String { String::from("db_value") }

let value = find_in_cache("user:123")
    .unwrap_or_else(|| fetch_from_db("user:123"));  // DB query only if cache miss
```

### map - Transform the Inner Value

This is where it gets interesting. Transform `Some(x)` to `Some(f(x))`, leave `None` alone:

```rust
impl<T> MyOption<T> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MyOption<U> {
        match self {
            Some(x) => Some(f(x)),
            None => None,
        }
    }
}
```

**Examples:**

```rust
// Basic transformation
let maybe_name: MyOption<String> = Some(String::from("alice"));
let maybe_len: MyOption<usize> = maybe_name.map(|s| s.len());
maybe_len  // Some(5)

let nothing: MyOption<String> = None;
let still_nothing: MyOption<usize> = nothing.map(|s| s.len());
still_nothing  // None

// Convert between types
let age: MyOption<u32> = Some(25);
let age_str = age.map(|n| n.to_string());
age_str  // MyOption<String>: Some("25")

// Chain transformations
let number = Some(5);
let result = number
    .map(|n| n * 2)      // Some(10)
    .map(|n| n + 3)      // Some(13)
    .map(|n| n.to_string());  // Some("13")
result  // Some("13")

// None propagates through
let number: MyOption<i32> = None;
let result = number
    .map(|n| n * 2)
    .map(|n| n + 3);
result  // None

// Working with structs
struct User {
    name: String,
    age: u32,
}

let user = Some(User {
    name: String::from("Alice"),
    age: 30,
});

let user_name = user.map(|u| u.name);
user_name  // Some("Alice")

// Real-world: parsing configuration
fn get_port_config() -> MyOption<String> {
    Some(String::from("8080"))
}

let port: MyOption<u16> = get_port_config()
    .map(|s| s.parse::<u16>().unwrap_or(3000));
port  // Some(8080)
```

### and_then - Chainable Operations (flatMap)

What if your transformation also returns an `Option`? `map` would give you `Option<Option<T>>`. Use `and_then` instead:

```rust
impl<T> MyOption<T> {
    fn and_then<U, F: FnOnce(T) -> MyOption<U>>(self, f: F) -> MyOption<U> {
        match self {
            Some(x) => f(x),
            None => None,
        }
    }
}
```

**How it works conceptually:**

The key insight: **unwrap self first, then apply f**

1. If `self` is `Some(x)`, unwrap it to get `x`, then apply `f(x)` which returns `MyOption<U>`
2. If `self` is `None`, just return `None` (no unwrapping needed)

This is different from `map`:

- `map(f)`: unwrap → apply f → **wrap result in Some**
- `and_then(f)`: unwrap → apply f → **return result as-is** (f already returns Option)

```rust
// Example: Why and_then avoids nesting
let x: MyOption<i32> = Some(5);

// With map: f returns MyOption, so we get nested Option
let nested = x.map(|n| Some(n * 2));  // MyOption<MyOption<i32>>

// With and_then: f returns MyOption, result stays flat
let flat = x.and_then(|n| Some(n * 2));  // MyOption<i32>
```

**Examples:**

```rust
// Why we need and_then: Compare map vs and_then
fn safe_divide(a: i32, b: i32) -> MyOption<i32> {
    if b == 0 {
        None
    } else {
        Some(a / b)
    }
}

// Using map: ❌ Gives nested Option
let x = Some(10);
let result = x.map(|n| safe_divide(n, 2));
// result = Some(Some(5)) - Wrong! We have nested Options

// Using and_then: ✅ Flattens automatically
let x = Some(10);
let result = x.and_then(|n| safe_divide(n, 2));
// result = Some(5) - Correct!
result  // Some(5)

// None propagates
let x = Some(10);
let result = x.and_then(|n| safe_divide(n, 0));
result  // None

// Processing multiple Options together
let a = Some(3);
let b = Some(2);

// Combine two Options: a + b
let sum = a.and_then(|x| b.map(|y| x + y));
sum  // Some(5)

// If either is None, result is None
let a = Some(3);
let b: MyOption<i32> = None;
let sum = a.and_then(|x| b.map(|y| x + y));
sum  // None

// Three Options: a + b + c
let a = Some(3);
let b = Some(2);
let c = Some(1);

let sum = a.and_then(|x|
    b.and_then(|y|
        c.map(|z| x + y + z)
    )
);
sum  // Some(6)

// Alternative: using match for multiple Options (often cleaner)
let a = Some(3);
let b = Some(2);

let sum = match (a, b) {
    (Some(x), Some(y)) => Some(x + y),
    _ => None,
};
sum  // Some(5)
```

### filter - Conditional Keep

Keep `Some` only if a predicate is true:

```rust
impl<T> MyOption<T> {
    fn filter<P: FnOnce(&T) -> bool>(self, predicate: P) -> MyOption<T> {
        match self {
            Some(x) if predicate(&x) => Some(x),
            _ => None,
        }
    }
}
```

**Examples:**

```rust
// Basic filtering
let even_number = Some(4).filter(|n| n % 2 == 0);
even_number  // Some(4)

let odd_number = Some(3).filter(|n| n % 2 == 0);
odd_number  // None

// None stays None
let nothing: MyOption<i32> = None;
let still_nothing = nothing.filter(|n| n % 2 == 0);
still_nothing  // None
```

### as_ref - Borrow the Inner Value

Convert `&MyOption<T>` to `MyOption<&T>`:

```rust
impl<T> MyOption<T> {
    fn as_ref(&self) -> MyOption<&T> {
        match self {
            Some(x) => Some(x),
            None => None,
        }
    }
}
```

**Why do we need this?** Because `map` takes `self` by value - it consumes the Option.

**Examples:**

```rust
// Problem: map consumes the Option
let maybe_name: MyOption<String> = Some(String::from("Alice"));
let len = maybe_name.map(|s| s.len());
// println!("{:?}", maybe_name);  // ERROR: maybe_name was moved!

// Solution: Use as_ref() to borrow
let maybe_name: MyOption<String> = Some(String::from("Alice"));
let len = maybe_name.as_ref().map(|s| s.len());  // s is &String
len  // Some(5)
println!("{:?}", maybe_name);  // Works! maybe_name still valid

// Multiple operations on the same Option
let data = Some(String::from("hello world"));

let len = data.as_ref().map(|s| s.len());
let uppercase = data.as_ref().map(|s| s.to_uppercase());
let contains = data.as_ref().map(|s| s.contains("world"));

len  // Some(11)
uppercase  // Some("HELLO WORLD")
contains  // Some(true)
// data is still usable!
data  // Some("hello world")

// as_ref with None
let nothing: MyOption<String> = None;
let result = nothing.as_ref().map(|s| s.len());
result  // None

// Real-world: Validating without consuming
struct Config {
    api_key: MyOption<String>,
}

impl Config {
    fn validate(&self) -> bool {
        // Use as_ref to check without consuming api_key
        self.api_key
            .as_ref()
            .map(|key| key.len() > 10)
            .unwrap_or(false)
    }

    fn get_key(&self) -> MyOption<&str> {
        // Convert MyOption<String> to MyOption<&str>
        self.api_key.as_ref().map(|s| s.as_str())
    }
}

let config = Config {
    api_key: Some(String::from("secret_key_12345")),
};

config.validate()  // true (borrows api_key)
config.validate()  // true (can validate again!)
config.get_key()  // Some("secret_key_12345")

// Chaining with as_ref
let text = Some(String::from("  hello  "));
let trimmed_len = text
    .as_ref()
    .map(|s| s.trim())
    .map(|s| s.len());
trimmed_len  // Some(5)
text  // Some("  hello  ") - original unchanged
```

The key insight: `as_ref()` converts `&MyOption<T>` to `MyOption<&T>`. Now when `map` consumes the Option, it's consuming an Option of _references_, not the original data.

### take - Extract and Replace with None

Useful for moving values out of mutable references:

```rust
impl<T> MyOption<T> {
    fn take(&mut self) -> MyOption<T> {
        std::mem::replace(self, None)
    }
}
```

**Examples:**

```rust
// Basic usage: Move value out, leave None
let mut slot: MyOption<String> = Some(String::from("hello"));
let taken = slot.take();

taken  // Some("hello")
slot  // None (slot is now None)

// Taking from None returns None
let mut empty: MyOption<i32> = None;
let result = empty.take();
result  // None
empty  // None

// Use case: Moving from struct fields
struct Cache {
    data: MyOption<String>,
}

impl Cache {
    fn flush(&mut self) -> MyOption<String> {
        // Take the data, leaving cache empty
        self.data.take()
    }

    fn get(&self) -> MyOption<&str> {
        // Use as_ref for non-destructive access
        self.data.as_ref().map(|s| s.as_str())
    }
}

let mut cache = Cache {
    data: Some(String::from("cached_value")),
};

cache.get()  // Some("cached_value")

let flushed = cache.flush();
flushed  // Some("cached_value")
cache.get()  // None (cache is now empty)

// Real-world: State machine transitions
struct Connection {
    state: MyOption<String>,
}

impl Connection {
    fn new() -> Self {
        Connection {
            state: Some(String::from("connected")),
        }
    }

    fn disconnect(&mut self) -> bool {
        // Take the state, disconnecting
        self.state.take().is_some()
    }

    fn is_connected(&self) -> bool {
        self.state.is_some()
    }
}

let mut conn = Connection::new();
conn.is_connected()  // true

let was_connected = conn.disconnect();
was_connected  // true
conn.is_connected()  // false

// Taking in a loop
let mut items = vec![
    Some(1),
    Some(2),
    Some(3),
];

let extracted: Vec<i32> = items
    .iter_mut()
    .filter_map(|opt| opt.take())
    .collect();

extracted  // [1, 2, 3]
// All items are now None
items.iter().all(|opt| opt.is_none())  // true

// Conditional take
struct Player {
    weapon: MyOption<String>,
}

impl Player {
    fn drop_weapon_if(&mut self, condition: bool) -> MyOption<String> {
        if condition {
            self.weapon.take()
        } else {
            None
        }
    }
}

let mut player = Player {
    weapon: Some(String::from("sword")),
};

// Don't drop
let result = player.drop_weapon_if(false);
result  // None
player.weapon  // Some("sword")

// Do drop
let result = player.drop_weapon_if(true);
result  // Some("sword")
player.weapon  // None
```

## The Complete Implementation

```rust
#[derive(Debug, Clone, PartialEq)]
enum MyOption<T> {
    Some(T),
    None,
}

use MyOption::{Some, None};

impl<T> MyOption<T> {
    fn is_some(&self) -> bool {
        matches!(self, Some(_))
    }

    fn is_none(&self) -> bool {
        !self.is_some()
    }

    fn unwrap(self) -> T {
        match self {
            Some(val) => val,
            None => panic!("called unwrap on a None value"),
        }
    }

    fn unwrap_or(self, default: T) -> T {
        match self {
            Some(val) => val,
            None => default,
        }
    }

    fn unwrap_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Some(val) => val,
            None => f(),
        }
    }

    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MyOption<U> {
        match self {
            Some(x) => MyOption::Some(f(x)),
            None => MyOption::None,
        }
    }

    fn and_then<U, F: FnOnce(T) -> MyOption<U>>(self, f: F) -> MyOption<U> {
        match self {
            Some(x) => f(x),
            None => MyOption::None,
        }
    }

    fn filter<P: FnOnce(&T) -> bool>(self, predicate: P) -> MyOption<T> {
        match self {
            Some(x) if predicate(&x) => Some(x),
            _ => None,
        }
    }

    fn as_ref(&self) -> MyOption<&T> {
        match self {
            Some(x) => MyOption::Some(x),
            None => MyOption::None,
        }
    }

    fn take(&mut self) -> MyOption<T> {
        std::mem::replace(self, None)
    }
}
```

## Key Takeaways

1. **Option is just an enum** - No magic, just two variants
2. **The compiler enforces handling** - Can't ignore the `None` case
3. **map transforms, and_then chains** - Functional programming patterns
4. **unwrap is a code smell** - Prefer `unwrap_or`, `unwrap_or_else`, or pattern matching

## Exercises

1. Implement `or` - returns self if `Some`, otherwise returns the other option
2. Implement `or_else` - lazy version of `or`
3. Implement `zip` - combines two options into one `Option<(T, U)>`
4. Implement `flatten` - converts `MyOption<MyOption<T>>` into `MyOption<T>`

## Next Chapter

[Result](./02-result.md) - Like Option, but with error information.
