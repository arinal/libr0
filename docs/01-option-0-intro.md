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

In Rust, to represent "no value," we use an enum called `Option`, which we'll implement ourselves as `Option0`.

## Our Option Type

```rust
enum Option0<T> {
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
use Option0::{Some, None};

fn find_user(id: u32) -> Option0<String> {
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

Notice that `find_user` returns `Option0<String>`, not `String`. This is the key difference:

| With null (Java, etc.)                   | With Option (Rust)                     |
| ---------------------------------------- | -------------------------------------- |
| `String find_user(...)`                  | `Option0<String> find_user(...)`      |
| Return type lies - might be null         | Return type is honest - might be None  |
| Compiler lets you ignore null            | Compiler **forces** you to handle None |
| Crash at runtime: `NullPointerException` | Error at compile time                  |

```java
// Java: Compiler is happy, but this crashes at runtime
String user = findUser(99);
int len = user.length();  // NullPointerException!
```

```rust
// Rust: Compiler is not happy, `user` is Option0<String>, not String
let user = find_user(99);
let len = user.len();  // Error: Option0<String> has no method `len`
```

```rust
// You MUST unwrap it first, which forces you to think about None
let len = match user {
    Some(s) => s.len(),
    None => 0,  // You're forced to decide what happens here
};
```

The compiler is your safety net. It won't let you forget.