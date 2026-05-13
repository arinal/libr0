# Unwrapping & Defaults

### is_ok and is_err

```rust
impl<T, E> Result0<T, E> {
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
let success: Result0<i32, &str> = Ok(42);
success.is_ok()   // true
success.is_err()  // false

let failure: Result0<i32, &str> = Err("bad input");
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
impl<T, E: std::fmt::Debug> Result0<T, E> {
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
let success: Result0<i32, &str> = Ok(42);
success.unwrap()  // 42

let failure: Result0<i32, &str> = Err("oops");
failure.unwrap()  // ❌ Panics: "called unwrap on Err: \"oops\""

// expect provides context
let result: Result0<Config, &str> = Err("missing file");
result.expect("Config must be loaded");
// ❌ Panics: "Config must be loaded: \"missing file\""

// Anti-pattern: checking then unwrapping
let result: Result0<i32, &str> = Ok(42);
if result.is_ok() {
    let val = result.unwrap();  // Won't panic, but verbose and clunky
    // use val...
}
// What about the Err case? You still need another if/else!

// Pattern matching is cleaner - extracts value and handles both cases
let result: Result0<i32, &str> = Ok(42);
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
impl<T, E> Result0<T, E> {
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
let success: Result0<i32, &str> = Ok(10);
success.unwrap_or(0)  // 10

let failure: Result0<i32, &str> = Err("bad");
failure.unwrap_or(0)  // 0

let result: Result0<i32, &str> = Err("parse error");
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
