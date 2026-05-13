# Transforming & Chaining

### map - Transform Success

Transform the `Ok` value, leave `Err` unchanged:

```rust
impl<T, E> Result0<T, E> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Result0<U, E> {
        match self {
            Ok(x) => Ok(f(x)),
            Err(e) => Err(e),
        }
    }
}
```

**Examples:**

```rust
let success: Result0<i32, &str> = Ok(5);
success.map(|x| x * 2)  // Ok(10)

let failure: Result0<i32, &str> = Err("bad");
failure.map(|x| x * 2)  // Err("bad") - unchanged!

// Misconception: map transforms both Ok and Err
// ❌ Wrong! map ONLY transforms Ok values
let result: Result0<i32, &str> = Err("error");
result.map(|x| x.to_string())  // Still Err("error"), not transformed

// Chain transformations
Ok(5).map(|x| x * 2).map(|x| x + 1)  // Ok(11)
```

### map_err - Transform Error

Transform the `Err` value, leave `Ok` unchanged:

```rust
impl<T, E> Result0<T, E> {
    fn map_err<F2, O: FnOnce(E) -> F2>(self, op: O) -> Result0<T, F2> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => Err(op(e)),
        }
    }
}
```

**Examples:**

```rust
let success: Result0<i32, &str> = Ok(5);
success.map_err(|e| e.to_uppercase())  // Ok(5) - unchanged!

let failure: Result0<i32, &str> = Err("bad");
failure.map_err(|e| e.to_uppercase())  // Err("BAD")

// map_err ONLY transforms Err values
Ok(42).map_err(String::from)  // Still Ok(42), not transformed

// Convert error types
#[derive(Debug)]
enum AppError { IoError(String), ParseError(String) }

let result: Result0<i32, &str> = Err("file not found");
result.map_err(|e| AppError::IoError(e.to_string()))  // Err(AppError::IoError(...))
```

### and_then - Chain Fallible Operations

The most important combinator. Chain operations that might fail:

```rust
impl<T, E> Result0<T, E> {
    fn and_then<U, F: FnOnce(T) -> Result0<U, E>>(self, f: F) -> Result0<U, E> {
        match self {
            Ok(x) => f(x),
            Err(e) => Err(e),
        }
    }
}
```

**Examples:**

```rust
fn safe_divide(a: i32, b: i32) -> Result0<i32, &'static str> {
    if b == 0 {
        Err("division by zero")
    } else {
        Ok(a / b)
    }
}

// Misconception: use map for Result-returning functions
let x: Result0<i32, &str> = Ok(10);
// x.map(|n| safe_divide(n, 2))  // ❌ Result0<Result0<i32, &str>, &str> - nested!

// ✅ Use and_then to avoid nesting
x.and_then(|n| safe_divide(n, 2))  // Result0<i32, &str> - flattened

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
impl<T, E> Result0<T, E> {
    fn ok(self) -> Option0<T> {
        match self {
            Ok(x) => Option0::Some(x),
            Err(_) => Option0::None,
        }
    }

    fn err(self) -> Option0<E> {
        match self {
            Ok(_) => Option0::None,
            Err(e) => Option0::Some(e),
        }
    }
}
```

**Examples:**

```rust
// ok() - Extract success value, discard error type
let success: Result0<i32, &str> = Ok(42);
success.ok()  // Some(42)

let failure: Result0<i32, &str> = Err("something went wrong");
failure.ok()  // None - error information lost!

// ✅ Use ok() when you don't care about the error
let port = parse_port("8080")
    .ok()
    .unwrap_or(3000);  // Use default if parse fails, don't care why

// err() - Extract error value, discard success value
let success: Result0<i32, &str> = Ok(42);
success.err()  // None

let failure: Result0<i32, &str> = Err("bad input");
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

Convert `&Result0<T, E>` to `Result0<&T, &E>`:

```rust
impl<T, E> Result0<T, E> {
    fn as_ref(&self) -> Result0<&T, &E> {
        match self {
            Ok(x) => Result0::Ok(x),
            Err(e) => Result0::Err(e),
        }
    }
}
```

**Examples:**

```rust
// Problem: map consumes the Result
let result: Result0<String, String> = Ok(String::from("hello"));
let len = result.map(|s| s.len());
// println!("{:?}", result);  // ❌ result was moved!

// ✅ Solution: Use as_ref() to borrow
let result: Result0<String, String> = Ok(String::from("hello"));
let len = result.as_ref().map(|s| s.len());  // s is &String
len  // Ok(5)
println!("{:?}", result);  // ✅ Works! result still valid

// Multiple operations on the same Result
let data: Result0<String, &str> = Ok(String::from("test"));

let len = data.as_ref().map(|s| s.len());
let uppercase = data.as_ref().map(|s| s.to_uppercase());
let is_empty = data.as_ref().map(|s| s.is_empty());

len  // Ok(4)
uppercase  // Ok("TEST")
is_empty  // Ok(false)
// data is still usable!

// Works with errors too
let failure: Result0<i32, String> = Err(String::from("error"));
let borrowed = failure.as_ref();  // Result0<&i32, &String>
borrowed  // Err(&String::from("error"))
```

