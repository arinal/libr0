# Transforming Options

## map - Transform the Inner Value

This is where it gets interesting. Transform `Some(x)` to `Some(f(x))`, leave `None` alone:

```rust
impl<T> Option0<T> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option0<U> {
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
let maybe_name: Option0<String> = Some(String::from("alice"));
let maybe_len: Option0<usize> = maybe_name.map(|s| s.len());
maybe_len  // Some(5)

let nothing: Option0<String> = None;
let still_nothing: Option0<usize> = nothing.map(|s| s.len());
still_nothing  // None

// Convert between types
let age: Option0<u32> = Some(25);
let age_str = age.map(|n| n.to_string());
age_str  // Option0<String>: Some("25")

// Chain transformations
let number = Some(5);
let result = number
    .map(|n| n * 2)      // Some(10)
    .map(|n| n + 3)      // Some(13)
    .map(|n| n.to_string());  // Some("13")
result  // Some("13")

// None propagates through
let number: Option0<i32> = None;
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
fn get_port_config() -> Option0<String> {
    Some(String::from("8080"))
}

let port: Option0<u16> = get_port_config()
    .map(|s| s.parse::<u16>().unwrap_or(3000));
port  // Some(8080)
```

## and_then - Chainable Operations (flatMap)

What if your transformation also returns an `Option`? `map` would give you `Option<Option<T>>`. Use `and_then` instead:

```rust
impl<T> Option0<T> {
    fn and_then<U, F: FnOnce(T) -> Option0<U>>(self, f: F) -> Option0<U> {
        match self {
            Some(x) => f(x),
            None => None,
        }
    }
}
```

**How it works conceptually:**

The key insight: **unwrap self first, then apply f**

1. If `self` is `Some(x)`, unwrap it to get `x`, then apply `f(x)` which returns `Option0<U>`
2. If `self` is `None`, just return `None` (no unwrapping needed)

This is different from `map`:

- `map(f)`: unwrap → apply f → **wrap result in Some**
- `and_then(f)`: unwrap → apply f → **return result as-is** (f already returns Option)

```rust
// Example: Why and_then avoids nesting
let x: Option0<i32> = Some(5);

// With map: f returns Option0, so we get nested Option
let nested = x.map(|n| Some(n * 2));  // Option0<Option0<i32>>

// With and_then: f returns Option0, result stays flat
let flat = x.and_then(|n| Some(n * 2));  // Option0<i32>
```

**Examples:**

```rust
// Why we need and_then: Compare map vs and_then
fn safe_divide(a: i32, b: i32) -> Option0<i32> {
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
let b: Option0<i32> = None;
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

## filter - Conditional Keep

Keep `Some` only if a predicate is true:

```rust
impl<T> Option0<T> {
    fn filter<P: FnOnce(&T) -> bool>(self, predicate: P) -> Option0<T> {
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
let nothing: Option0<i32> = None;
let still_nothing = nothing.filter(|n| n % 2 == 0);
still_nothing  // None
```