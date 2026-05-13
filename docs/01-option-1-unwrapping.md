# Unwrapping & Defaults

Let's build the most useful methods step by step.

## is_some and is_none

The simplest methods - just check which variant we have:

```rust
impl<T> Option0<T> {
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
let x: Option0<u32> = Some(42);
x.is_some()  // true
x.is_none()  // false

let y: Option0<u32> = None;
y.is_none()  // true
y.is_some()  // false

// Useful for conditional checks
if x.is_some() {
    println!("x has a value");
}

// Or for early returns
fn process(opt: Option0<i32>) -> Result<(), String> {
    if opt.is_none() {
        return Err("No value provided".to_string());
    }
    // Continue processing...
    Ok(())
}
```

## unwrap - The Dangerous One

Extract the value, panic if `None`:

```rust
impl<T> Option0<T> {
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
let y: Option0<&str> = None;
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

## unwrap_or - Safe Default

Provide a fallback value:

```rust
impl<T> Option0<T> {
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

let y: Option0<i32> = None;
y.unwrap_or(0)  // 0

// User input with fallback
fn get_count(user_input: Option0<i32>) -> i32 {
    user_input.unwrap_or(10)  // Default to 10 if no input
}

get_count(Some(5))  // 5
get_count(None)     // 10
```

## unwrap_or_else - Lazy Default

Sometimes computing the default is expensive. Only compute it if needed:

```rust
impl<T> Option0<T> {
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

let y: Option0<i32> = None;
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

let none_value: Option0<String> = None;
let result = none_value.unwrap_or_else(|| expensive_computation());
// "Computing..." IS printed because we have None
result  // "default"

// Database lookup as fallback
fn find_in_cache(key: &str) -> Option0<String> { None }
fn fetch_from_db(key: &str) -> String { String::from("db_value") }

let value = find_in_cache("user:123")
    .unwrap_or_else(|| fetch_from_db("user:123"));  // DB query only if cache miss
```