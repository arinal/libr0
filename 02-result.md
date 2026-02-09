# Chapter 2: Result - Error Handling Done Right

## The Problem: Exceptions Are Invisible

In many languages, any function can throw an exception:

```java
String content = readFile("config.txt"); // might throw IOException
int port = Integer.parseInt(content);     // might throw NumberFormatException
```

The compiler doesn't force you to handle these. Errors become invisible landmines in your code. Java's checked exceptions tried to fix this, but they're widely considered a failed experiment.

Rust's approach: make errors **visible in the type signature**.

## Our Result Type

```rust
enum MyResult<T, E> {
    Ok(T),
    Err(E),
}
```

Two variants:

- `Ok(T)` - operation succeeded with value `T`
- `Err(E)` - operation failed with error `E`

The caller **must** handle both cases. The compiler won't let you ignore errors.

## Basic Usage

```rust
use MyResult::{Ok, Err};

#[derive(Debug)]
enum ParseError {
    Empty,
    InvalidNumber,
}

fn parse_port(s: &str) -> MyResult<u16, ParseError> {
    if s.is_empty() {
        return Err(ParseError::Empty);
    }

    match s.parse() {
        std::result::Result::Ok(n) => Ok(n),
        std::result::Result::Err(_) => Err(ParseError::InvalidNumber),
    }
}

fn main() {
    match parse_port("8080") {
        Ok(port) => println!("Port: {}", port),
        Err(e) => println!("Error: {:?}", e),
    }
}
```

## Implementing Result Methods

### is_ok and is_err

```rust
impl<T, E> MyResult<T, E> {
    fn is_ok(&self) -> bool {
        matches!(self, Ok(_))
    }

    fn is_err(&self) -> bool {
        !self.is_ok()
    }
}
```

**Examples:**

```rust
let success: MyResult<i32, &str> = Ok(42);
success.is_ok()   // true
success.is_err()  // false

let failure: MyResult<i32, &str> = Err("bad input");
failure.is_ok()   // false
failure.is_err()  // true

// ❌ Common mistake: verbose pattern
if result.is_ok() {
    let value = result.unwrap();  // Don't do this!
    // use value...
}

// ✅ Better: use match or if let
match result {
    Ok(value) => { /* use value */ },
    Err(e) => { /* handle error */ }
}
```

### unwrap and expect

Extract value, panic on error:

```rust
impl<T, E: std::fmt::Debug> MyResult<T, E> {
    fn unwrap(self) -> T {
        match self {
            Ok(val) => val,
            Err(e) => panic!("called unwrap on Err: {:?}", e),
        }
    }

    fn expect(self, msg: &str) -> T {
        match self {
            Ok(val) => val,
            Err(e) => panic!("{}: {:?}", msg, e),
        }
    }
}
```

`expect` is slightly better than `unwrap` - at least you leave a message explaining what went wrong.

**Examples:**

```rust
let success: MyResult<i32, &str> = Ok(42);
success.unwrap()  // 42

let failure: MyResult<i32, &str> = Err("oops");
// failure.unwrap()  // ❌ Panics: "called unwrap on Err: \"oops\""

// expect provides context
let result: MyResult<Config, &str> = Err("missing file");
// result.expect("Config must be loaded");
// ❌ Panics: "Config must be loaded: \"missing file\""

// Misconception: "I checked is_ok(), so unwrap is safe"
if result.is_ok() {
    let val = result.unwrap();  // ❌ result was moved by is_ok()!
}

// Use pattern matching instead
match result {
    Ok(val) => { /* use val */ },
    Err(e) => { /* handle e */ }
}
```

### unwrap_or and unwrap_or_else

```rust
impl<T, E> MyResult<T, E> {
    fn unwrap_or(self, default: T) -> T {
        match self {
            Ok(val) => val,
            Err(_) => default,
        }
    }

    fn unwrap_or_else<F: FnOnce(E) -> T>(self, f: F) -> T {
        match self {
            Ok(val) => val,
            Err(e) => f(e),
        }
    }
}
```

**Examples:**

```rust
let success: MyResult<i32, &str> = Ok(10);
success.unwrap_or(0)  // 10

let failure: MyResult<i32, &str> = Err("bad");
failure.unwrap_or(0)  // 0

// Misconception: unwrap_or_else doesn't get the error
let result: MyResult<i32, &str> = Err("parse error");
let val = result.unwrap_or_else(|e| {
    eprintln!("Error: {}", e);  // ✅ Has access to error!
    0
});

// Key difference: unwrap_or vs unwrap_or_else
fn expensive_default() -> i32 {
    println!("Computing default...");
    42
}

Ok(10).unwrap_or(expensive_default())       // Calls expensive_default() even for Ok!
Ok(10).unwrap_or_else(|_| expensive_default())  // ✅ Only calls on Err
```

### map - Transform Success

Transform the `Ok` value, leave `Err` unchanged:

```rust
impl<T, E> MyResult<T, E> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MyResult<U, E> {
        match self {
            Ok(x) => Ok(f(x)),
            Err(e) => Err(e),
        }
    }
}
```

**Examples:**

```rust
let success: MyResult<i32, &str> = Ok(5);
success.map(|x| x * 2)  // Ok(10)

let failure: MyResult<i32, &str> = Err("bad");
failure.map(|x| x * 2)  // Err("bad") - unchanged!

// Misconception: map transforms both Ok and Err
// ❌ Wrong! map ONLY transforms Ok values
let result: MyResult<i32, &str> = Err("error");
result.map(|x| x.to_string())  // Still Err("error"), not transformed

// Chain transformations
Ok(5).map(|x| x * 2).map(|x| x + 1)  // Ok(11)
```

### map_err - Transform Error

Transform the `Err` value, leave `Ok` unchanged:

```rust
impl<T, E> MyResult<T, E> {
    fn map_err<F2, O: FnOnce(E) -> F2>(self, op: O) -> MyResult<T, F2> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(op(e)),
        }
    }
}
```

**Examples:**

```rust
let success: MyResult<i32, &str> = Ok(5);
success.map_err(|e| e.to_uppercase())  // Ok(5) - unchanged!

let failure: MyResult<i32, &str> = Err("bad");
failure.map_err(|e| e.to_uppercase())  // Err("BAD")

// Misconception: map_err transforms Ok values
// ❌ Wrong! map_err ONLY transforms Err values
Ok(42).map_err(String::from)  // Still Ok(42), not transformed

// Convert error types
#[derive(Debug)]
enum AppError { IoError(String), ParseError(String) }

let result: MyResult<i32, &str> = Err("file not found");
result.map_err(|e| AppError::IoError(e.to_string()))  // Err(AppError::IoError(...))
```

### and_then - Chain Fallible Operations

The most important combinator. Chain operations that might fail:

```rust
impl<T, E> MyResult<T, E> {
    fn and_then<U, F: FnOnce(T) -> MyResult<U, E>>(self, f: F) -> MyResult<U, E> {
        match self {
            Ok(x) => f(x),
            Err(e) => Err(e),
        }
    }
}
```

**Examples:**

```rust
fn safe_divide(a: i32, b: i32) -> MyResult<i32, &'static str> {
    if b == 0 {
        Err("division by zero")
    } else {
        Ok(a / b)
    }
}

// Misconception: use map for Result-returning functions
let x: MyResult<i32, &str> = Ok(10);
// x.map(|n| safe_divide(n, 2))  // ❌ MyResult<MyResult<i32, &str>, &str> - nested!

// ✅ Use and_then to avoid nesting
x.and_then(|n| safe_divide(n, 2))  // MyResult<i32, &str> - flattened

// Chain multiple fallible operations
Ok(20)
    .and_then(|n| safe_divide(n, 2))   // Ok(10)
    .and_then(|n| safe_divide(n, 5))   // Ok(2)

// Errors propagate
Ok(10)
    .and_then(|n| safe_divide(n, 0))  // Err("division by zero")
    .and_then(|n| safe_divide(n, 2))  // Still Err, second operation skipped
```

### ok - Convert to Option

Discard the error, convert to `Option`:

```rust
impl<T, E> MyResult<T, E> {
    fn ok(self) -> MyOption<T> {
        match self {
            Ok(x) => MyOption::Some(x),
            Err(_) => MyOption::None,
        }
    }

    fn err(self) -> MyOption<E> {
        match self {
            Ok(_) => MyOption::None,
            Err(e) => MyOption::Some(e),
        }
    }
}
```

**Examples:**

```rust
// ok() - Extract success value, discard error type
let success: MyResult<i32, &str> = Ok(42);
success.ok()  // Some(42)

let failure: MyResult<i32, &str> = Err("something went wrong");
failure.ok()  // None - error information lost!

// Misconception: ok() preserves error information
// ❌ Wrong! ok() discards the error
let result: MyResult<i32, String> = Err(String::from("detailed error"));
let option = result.ok();  // None - "detailed error" is gone

// ✅ Use ok() when you don't care about the error
let port = parse_port("8080")
    .ok()
    .unwrap_or(3000);  // Use default if parse fails, don't care why

// err() - Extract error value, discard success value
let success: MyResult<i32, &str> = Ok(42);
success.err()  // None

let failure: MyResult<i32, &str> = Err("bad input");
failure.err()  // Some("bad input")

// Use case: Collecting errors
let results = vec![Ok(1), Err("error1"), Ok(2), Err("error2")];
let errors: Vec<&str> = results
    .into_iter()
    .filter_map(|r| r.err())
    .collect();
errors  // ["error1", "error2"]
```

### as_ref - Borrow the Inner Values

Convert `&MyResult<T, E>` to `MyResult<&T, &E>`:

```rust
impl<T, E> MyResult<T, E> {
    fn as_ref(&self) -> MyResult<&T, &E> {
        match self {
            Ok(x) => MyResult::Ok(x),
            Err(e) => MyResult::Err(e),
        }
    }
}
```

**Examples:**

```rust
// Problem: map consumes the Result
let result: MyResult<String, String> = Ok(String::from("hello"));
let len = result.map(|s| s.len());
// println!("{:?}", result);  // ❌ ERROR: result was moved!

// ✅ Solution: Use as_ref() to borrow
let result: MyResult<String, String> = Ok(String::from("hello"));
let len = result.as_ref().map(|s| s.len());  // s is &String
len  // Ok(5)
println!("{:?}", result);  // ✅ Works! result still valid

// Multiple operations on the same Result
let data: MyResult<String, &str> = Ok(String::from("test"));

let len = data.as_ref().map(|s| s.len());
let uppercase = data.as_ref().map(|s| s.to_uppercase());
let is_empty = data.as_ref().map(|s| s.is_empty());

len  // Ok(4)
uppercase  // Ok("TEST")
is_empty  // Ok(false)
// data is still usable!

// Works with errors too
let failure: MyResult<i32, String> = Err(String::from("error"));
let borrowed = failure.as_ref();  // MyResult<&i32, &String>
borrowed  // Err(&String::from("error"))
```

## The ? Operator

Chaining with `and_then` works, but gets verbose:

```rust
fn process_config() -> MyResult<Config, Error> {
    read_file("config.txt")
        .and_then(|content| parse_config(&content))
        .and_then(|raw| validate_config(raw))
        .and_then(|valid| apply_defaults(valid))
}
```

Rust's `?` operator makes this cleaner:

```rust
fn process_config() -> Result<Config, Error> {
    let content = read_file("config.txt")?;
    let raw = parse_config(&content)?;
    let valid = validate_config(raw)?;
    apply_defaults(valid)
}
```

The `?` operator is syntax sugar. This:

```rust
let content = read_file("config.txt")?;
```

...expands to roughly this:

```rust
let content = match read_file("config.txt") {
    Ok(val) => val,           // Unwrap and continue
    Err(e) => return Err(e),  // Early return with error
};
```

So the whole function:

```rust
fn process_config() -> Result<Config, Error> {
    let content = read_file("config.txt")?;
    let raw = parse_config(&content)?;
    apply_defaults(raw)
}
```

...is equivalent to:

```rust
fn process_config() -> Result<Config, Error> {
    let content = match read_file("config.txt") {
        Ok(val) => val,
        Err(e) => return Err(e),
    };
    let raw = match parse_config(&content) {
        Ok(val) => val,
        Err(e) => return Err(e),
    };
    apply_defaults(raw)
}
```

We can't implement `?` for our custom type (it requires the `Try` trait which is unstable), but understanding what it does is essential.

## Result vs Option

| Situation                      | Use            |
| ------------------------------ | -------------- |
| Value might not exist          | `Option<T>`    |
| Operation might fail           | `Result<T, E>` |
| Need to know why it failed     | `Result<T, E>` |
| Don't care about error details | `Option<T>`    |

Converting between them:

```rust
// Option -> Result
impl<T> MyOption<T> {
    fn ok_or<E>(self, err: E) -> MyResult<T, E> {
        match self {
            MyOption::Some(x) => MyResult::Ok(x),
            MyOption::None => MyResult::Err(err),
        }
    }

    fn ok_or_else<E, F: FnOnce() -> E>(self, f: F) -> MyResult<T, E> {
        match self {
            MyOption::Some(x) => MyResult::Ok(x),
            MyOption::None => MyResult::Err(f()),
        }
    }
}

// Result -> Option (already shown above with .ok())
```

## The Complete Implementation

```rust
#[derive(Debug, Clone, PartialEq)]
enum MyResult<T, E> {
    Ok(T),
    Err(E),
}

use MyResult::{Ok, Err};

impl<T, E> MyResult<T, E> {
    fn is_ok(&self) -> bool {
        matches!(self, Ok(_))
    }

    fn is_err(&self) -> bool {
        !self.is_ok()
    }

    fn ok(self) -> Option<T> {
        match self {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }

    fn err(self) -> Option<E> {
        match self {
            Ok(_) => None,
            Err(e) => Some(e),
        }
    }

    fn unwrap_or(self, default: T) -> T {
        match self {
            Ok(val) => val,
            Err(_) => default,
        }
    }

    fn unwrap_or_else<F: FnOnce(E) -> T>(self, f: F) -> T {
        match self {
            Ok(val) => val,
            Err(e) => f(e),
        }
    }

    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MyResult<U, E> {
        match self {
            Ok(x) => MyResult::Ok(f(x)),
            Err(e) => MyResult::Err(e),
        }
    }

    fn map_err<F2, O: FnOnce(E) -> F2>(self, op: O) -> MyResult<T, F2> {
        match self {
            Ok(x) => MyResult::Ok(x),
            Err(e) => MyResult::Err(op(e)),
        }
    }

    fn and_then<U, F: FnOnce(T) -> MyResult<U, E>>(self, f: F) -> MyResult<U, E> {
        match self {
            Ok(x) => f(x),
            Err(e) => MyResult::Err(e),
        }
    }

    fn as_ref(&self) -> MyResult<&T, &E> {
        match self {
            Ok(x) => MyResult::Ok(x),
            Err(e) => MyResult::Err(e),
        }
    }
}

impl<T, E: std::fmt::Debug> MyResult<T, E> {
    fn unwrap(self) -> T {
        match self {
            Ok(val) => val,
            Err(e) => panic!("called unwrap on Err: {:?}", e),
        }
    }

    fn expect(self, msg: &str) -> T {
        match self {
            Ok(val) => val,
            Err(e) => panic!("{}: {:?}", msg, e),
        }
    }
}
```

## Key Takeaways

1. **Errors are values** - Not hidden control flow like exceptions
2. **The type signature tells the truth** - `Result<T, E>` means "this can fail"
3. **map for success, map_err for errors** - Transform either side
4. **and_then chains fallible operations** - The workhorse of error handling
5. **? is syntax sugar for and_then + early return** - Use it in real code

## Exercises

1. Implement `or` - returns self if `Ok`, otherwise returns the other result
2. Implement `or_else` - lazy version with access to the error
3. Implement `and` - returns other if self is `Ok`, otherwise returns self's error
4. Implement `flatten` - converts `MyResult<MyResult<T, E>, E>` to `MyResult<T, E>`

## Next Chapter

[Box](./03-box.md) - Heap allocation and the `Deref` trait.
