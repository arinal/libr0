# The ? Operator

Chaining with `and_then` works, but gets verbose:

```rust
fn process_config() -> Result0<Config, Error> {
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
fn calculate(input: &str) -> Result0<i32, &str> {
    parse_int(input)
        .and_then(|n| safe_divide(n, 2))
        .and_then(|n| check_positive(n))
        .map(|n| n * 10)
}
// If any step returns Err, the chain stops and returns that Err

// Nested pattern - same calculation, nested style (like Scala's for-comprehension):
fn calculate_nested(input: &str) -> Result0<i32, &str> {
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
fn calculate(input: &str) -> Result0<i32, &str> {
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
fn multi_step() -> Result0<i32, &str> {
    let a = step1()?;        // Ok(5)  - continues
    let b = step2(a)?;       // Err("failed") - returns immediately
    let c = step3(b)?;       // Never runs
    let d = step4(c)?;       // Never runs
    Ok(d)                    // Never runs
}
// Returns: Err("failed")

// Expanded to show what happens:
fn multi_step_expanded() -> Result0<i32, &str> {
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
impl<T> Option0<T> {
    fn ok_or<E>(self, err: E) -> Result0<T, E> {
        match self {
            Option0::Some(x) => Result0::Ok(x),
            Option0::None => Result0::Err(err),
        }
    }

    fn ok_or_else<E, F: FnOnce() -> E>(self, f: F) -> Result0<T, E> {
        match self {
            Option0::Some(x) => Result0::Ok(x),
            Option0::None => Result0::Err(f()),
        }
    }
}

// Result -> Option (already shown above with .ok())
```

## Implementation

See the full code in [`src/result.rs`](./src/result.rs) for the complete implementation of `Result0` with all methods.
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