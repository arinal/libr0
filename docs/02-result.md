# Chapter 2: Result - Error Handling Done Right

## The Problem: Exceptions Are Invisible

In many languages, any function can throw an exception:

```java
String content = readFile("non-existent-file.txt"); // throws exception
println("File content: " + content);
```

In Java, the above code compiles fine, even though the programmer "forgot" to handle exception.

Rust's approach: readFile returns a wrapper to indicate it can fail:
```rust
let result = readFile("non-existent-file.txt"); // returns Result<String, Error>
// result is not the content, but a wrapper that can be Ok(content) or Err(error)
// to extract the content, you're forced to handle both cases:
match result {
    Ok(content) => println!("File content: {}", content),
    Err(e) => println!("Failed to read file: {:?}", e),
}
// this way, the programmer can't "forget" to handle errors, as the case with the java example.
```

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

## What Can Be an Error?

The `E` in `Result<T, E>` can be **any type**. It doesn't need to implement `std::error::Error` or any special trait, as long as you wrap it in `Err()`.

```rust
// String as error
let error: MyResult<i32, String> = Err(String::from("something broke"));

// &str as error
let error: MyResult<i32, &str> = Err("file not found");

// Number as error code
let error: MyResult<i32, i32> = Err(404);

// Custom enum - most common in real code
#[derive(Debug)]
enum ParseError {
    Empty,
    TooLong,
    InvalidFormat,
}
let error: MyResult<i32, ParseError> = Err(ParseError::Empty);
```

**Key rule**: Always wrap your error in `Err()`. Don't return the error type directly:

```rust
// ❌ Wrong
fn parse(s: &str) -> MyResult<i32, &str> {
    if s.is_empty() {
        "empty string"  // ERROR: expected MyResult, found &str
    } else {
        Ok(42)
    }
}

// ✅ Correct
fn parse(s: &str) -> MyResult<i32, &str> {
    if s.is_empty() {
        Err("empty string")  // Wrapped in Err!
    } else {
        Ok(42)
    }
}
```

## Basic Usage

Let's validate a person with a custom error type:

```rust
use MyResult::{Ok, Err};

#[derive(Debug)]
struct Person {
    name: String,
    age: i32,
}

#[derive(Debug)]
enum InvalidPersonError {
    EmptyName,
    InvalidAge(i32),
}

fn validate_person(person: Person) -> MyResult<Person, InvalidPersonError> {
    if person.name.is_empty() {
        Err(InvalidPersonError::EmptyName)  // Wrap in Err!
    } else if person.age < 0 {
        Err(InvalidPersonError::InvalidAge(person.age))  // Capture the bad value
    } else {
        Ok(person)  // Wrap valid person in Ok!
    }
}

fn main() {
    let person = Person { name: String::from("Alice"), age: 30 };
    match validate_person(person) {
        Ok(valid_person) => println!("Valid person: {:?}", valid_person),
        Err(e) => println!("Invalid person: {:?}", e),
    }
    // Output: Valid person: Person { name: "Alice", age: 30 }

    let bad_person = Person { name: String::from(""), age: -5 };
    match validate_person(bad_person) {
        Ok(valid_person) => println!("Valid person: {:?}", valid_person),
        Err(e) => println!("Invalid person: {:?}", e),
    }
    // Output: Invalid person: EmptyName
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
failure.unwrap()  // ❌ Panics: "called unwrap on Err: \"oops\""

// expect provides context
let result: MyResult<Config, &str> = Err("missing file");
result.expect("Config must be loaded");
// ❌ Panics: "Config must be loaded: \"missing file\""

// Anti-pattern: checking then unwrapping
let result: MyResult<i32, &str> = Ok(42);
if result.is_ok() {
    let val = result.unwrap();  // Won't panic, but verbose and clunky
    // use val...
}
// What about the Err case? You still need another if/else!

// Pattern matching is cleaner - extracts value and handles both cases
let result: MyResult<i32, &str> = Ok(42);
match result {
    Ok(val) => { /* use val */ },
    Err(e) => { /* handle error */ }
}

// Or use if let for the Ok case only
if let Ok(val) = result {
    // use val...
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

let result = Ok(10);

// expensive_default() is being called
// even though the result is not used!
let out = result.unwrap_or(expensive_default())
// expensive_default() is only called if result is Err
// which in this case it is not, so we avoid the unnecessary computation!
let out = result.unwrap_or_else(|_| expensive_default())

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

// map_err ONLY transforms Err values
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
// println!("{:?}", result);  // ❌ result was moved!

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

### The `?` Operator is Also Monadic

Both `and_then` and `?` are **monadic operations** - they both short-circuit on errors, just in different styles.

**`and_then` - Functional style (expression-based):**

```rust
// Linear chain:
fn calculate(input: &str) -> MyResult<i32, &str> {
    parse_int(input)
        .and_then(|n| safe_divide(n, 2))
        .and_then(|n| check_positive(n))
        .map(|n| n * 10)
}
// If any step returns Err, the chain stops and returns that Err

// Nested pattern - same calculation, nested style (like Scala's for-comprehension):
fn calculate_nested(input: &str) -> MyResult<i32, &str> {
    parse_int(input).and_then(|n|
        safe_divide(n, 2).and_then(|n2|
            check_positive(n2).map(|n3| n3 * 10)
        )
    )
}
// Same calculation as linear chain, but nested. Demonstrates short-circuiting beautifully:
// If parse_int returns Err, the nested closures are NEVER invoked at all!
```

**`?` - Imperative style (statement-based):**

```rust
fn calculate(input: &str) -> MyResult<i32, &str> {
    let n = parse_int(input)?;           // Returns Err if parse fails
    let n = safe_divide(n, 2)?;          // Returns Err if divide fails
    let n = check_positive(n)?;          // Returns Err if check fails
    Ok(n * 10)
}
// If any step returns Err, the function returns early with that Err
```

Both do the same thing: **stop on first error and propagate it up**.

**Visualizing `?` short-circuit:**

```rust
fn multi_step() -> MyResult<i32, &str> {
    let a = step1()?;        // Ok(5)  - continues
    let b = step2(a)?;       // Err("failed") - returns immediately
    let c = step3(b)?;       // Never runs
    let d = step4(c)?;       // Never runs
    Ok(d)                    // Never runs
}
// Returns: Err("failed")

// Expanded to show what happens:
fn multi_step_expanded() -> MyResult<i32, &str> {
    let a = match step1() {
        Ok(val) => val,
        Err(e) => return Err(e),  // Early return
    };
    let b = match step2(a) {
        Ok(val) => val,
        Err(e) => return Err(e),  // Early return - stops here!
    };
    // Everything below never executes
    let c = match step3(b) {
        Ok(val) => val,
        Err(e) => return Err(e),
    };
    let d = match step4(c) {
        Ok(val) => val,
        Err(e) => return Err(e),
    };
    Ok(d)
}
```

**Key insight**: Both `and_then` and `?` implement the same monadic pattern:

1. Execute an operation that might fail
2. If it succeeds, continue with the result
3. If it fails, stop immediately and propagate the error

This is why Result-based error handling in Rust is so ergonomic - errors automatically bubble up without explicit checking at every step.

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

## Implementation

See the full code in [`src/result.rs`](./src/result.rs) for the complete implementation of `MyResult` with all methods.
Also, see the exercises in [02_result.rs](./examples/02_result.rs)

## Key Takeaways

1. **Errors are values** - Not hidden control flow like exceptions. The compiler forces you to handle them.
2. **The type signature tells the truth** - `Result<T, E>` means "this can fail". No surprises, no invisible exceptions.
3. **E can be any type** - String, &str, enums, integers, custom types. No special traits required. Just wrap it in `Err()`.
4. **map for success, map_err for errors** - Transform either side independently. Only one variant changes at a time.
5. **and_then chains fallible operations** - The workhorse of error handling. Flattens nested Results and short-circuits on first error.
6. **Two styles, same pattern** - Linear chains (`and_then`) and nested closures both demonstrate monadic short-circuiting. If any step fails, everything stops.
7. **? is syntax sugar for and_then + early return** - Imperative style that does the same thing. Use it in real code.
8. **Short-circuit behavior is free** - Errors automatically propagate up without explicit checking at every step. That's why Result-based error handling is so ergonomic.

## Next Chapter

[Box](./03-box.md) - Heap allocation and the `Deref` trait.
