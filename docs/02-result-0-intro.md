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
enum Result0<T, E> {
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
let error: Result0<i32, String> = Err(String::from("something broke"));

// &str as error
let error: Result0<i32, &str> = Err("file not found");

// Number as error code
let error: Result0<i32, i32> = Err(404);

// Custom enum - most common in real code
#[derive(Debug)]
enum ParseError {
    Empty,
    TooLong,
    InvalidFormat,
}
let error: Result0<i32, ParseError> = Err(ParseError::Empty);
```

**Key rule**: Always wrap your error in `Err()`. Don't return the error type directly:

```rust
// ❌ Wrong
fn parse(s: &str) -> Result0<i32, &str> {
    if s.is_empty() {
        "empty string"  // ERROR: expected Result0, found &str
    } else {
        Ok(42)
    }
}

// ✅ Correct
fn parse(s: &str) -> Result0<i32, &str> {
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
use Result0::{Ok, Err};

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

fn validate_person(person: Person) -> Result0<Person, InvalidPersonError> {
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

