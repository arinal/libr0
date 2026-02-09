//! Chapter 1: Option - The Simplest Enum
//!
//! Run with: cargo run --example option

#[derive(Debug, Clone, PartialEq)]
enum MyOption<T> {
    Some(T),
    None,
}

use MyOption::{None, Some};

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

    fn unwrap_or(self, or: T) -> T {
        match self {
            Some(val) => val,
            None => or,
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

    // Exercise solutions

    fn or(self, other: MyOption<T>) -> MyOption<T> {
        match self {
            Some(x) => Some(x),
            None => other,
        }
    }

    fn or_else<F: FnOnce() -> MyOption<T>>(self, f: F) -> MyOption<T> {
        match self {
            Some(x) => Some(x),
            None => f(),
        }
    }
}

impl<T, U> MyOption<(T, U)> {
    fn unzip(self) -> (MyOption<T>, MyOption<U>) {
        match self {
            Some((a, b)) => (Some(a), Some(b)),
            None => (None, None),
        }
    }
}

impl<T> MyOption<MyOption<T>> {
    fn flatten(self) -> MyOption<T> {
        match self {
            Some(inner) => inner,
            None => None,
        }
    }
}

// Zip requires a separate function since we need two type parameters
fn zip<T, U>(a: MyOption<T>, b: MyOption<U>) -> MyOption<(T, U)> {
    match (a, b) {
        (Some(x), Some(y)) => Some((x, y)),
        _ => None,
    }
}

// ============================================================================
// Demo
// ============================================================================

fn find_user(id: u32) -> MyOption<String> {
    match id {
        1 => Some(String::from("Alice")),
        2 => Some(String::from("Bob")),
        _ => None,
    }
}

fn get_user_email(name: &str) -> MyOption<String> {
    match name {
        "Alice" => Some(String::from("alice@example.com")),
        _ => None,
    }
}

fn _01_basic_usage() {
    println!("--- Basic Usage ---");
    let user = find_user(1);
    match &user {
        Some(name) => println!("Found user: {}", name),
        None => println!("User not found"),
    }
}

fn _02_is_some_is_none() {
    println!("\n--- is_some / is_none ---");
    let x: MyOption<u32> = Some(42);
    println!("Some(42).is_some() = {}", x.is_some());
    println!("Some(42).is_none() = {}", x.is_none());

    let y: MyOption<u32> = None;
    println!("None.is_some() = {}", y.is_some());
    println!("None.is_none() = {}", y.is_none());

    // Early return pattern
    if find_user(99).is_none() {
        println!("User 99 not found - would return early in real code");
    }
}

fn _03_unwrap() {
    println!("\n--- unwrap (use with caution!) ---");
    let x = Some("value");
    println!("Some(\"value\").unwrap() = {}", x.unwrap());
    println!("Note: unwrap on None would panic! Only use when 100% sure it's Some");
}

fn _04_unwrap_or() {
    println!("\n--- unwrap_or ---");
    let x = Some(42);
    println!("Some(42).unwrap_or(0) = {}", x.unwrap_or(0));

    let y: MyOption<i32> = None;
    println!("None.unwrap_or(0) = {}", y.unwrap_or(0));

    // Configuration with defaults
    let port = None.unwrap_or(8080);
    println!("Default port: {}", port);
}

fn _05_unwrap_or_else() {
    println!("\n--- unwrap_or_else (lazy evaluation) ---");
    let x = Some(42);
    println!("Some(42).unwrap_or_else(|| 0) = {}", x.unwrap_or_else(|| {
        println!("  (This closure is NOT called for Some)");
        0
    }));

    let y: MyOption<i32> = None;
    println!("None.unwrap_or_else(|| 0) = {}", y.unwrap_or_else(|| {
        println!("  (This closure IS called for None)");
        0
    }));
}

fn _06_map() {
    println!("\n--- map (transform values) ---");
    let name = Some(String::from("alice"));
    let name_len = name.map(|s| s.len());
    println!("Some(\"alice\").map(len) = {:?}", name_len);

    let nothing: MyOption<String> = None;
    let nothing_len = nothing.map(|s| s.len());
    println!("None.map(len) = {:?}", nothing_len);

    // Chaining maps
    let number = Some(5);
    let result = number
        .map(|n| n * 2)
        .map(|n| n + 3)
        .map(|n| n.to_string());
    println!("Some(5) -> *2 -> +3 -> to_string = {:?}", result);

    // None propagates through map chain
    let none_number: MyOption<i32> = None;
    let none_result = none_number.map(|n| n * 2).map(|n| n + 3);
    println!("None -> map -> map = {:?}", none_result);
}

fn _07_and_then() {
    println!("\n--- and_then (chain fallible operations) ---");

    // Compare map vs and_then
    println!("Why and_then? Because map would give nested Options:");
    let _x = Some(10);
    // Using map would give Some(Some(5)) - nested!
    // Using and_then gives Some(5) - flattened!

    let email = find_user(1).and_then(|name| get_user_email(&name));
    println!("find_user(1) -> get_email = {:?}", email);

    let no_email = find_user(2).and_then(|name| get_user_email(&name));
    println!("find_user(2) -> get_email = {:?} (Bob has no email)", no_email);

    let no_user = find_user(99).and_then(|name| get_user_email(&name));
    println!("find_user(99) -> get_email = {:?} (user not found)", no_user);
}

fn _08_filter() {
    println!("\n--- filter (conditional keep) ---");

    // Keep even numbers
    let even = Some(4).filter(|n| n % 2 == 0);
    println!("Some(4).filter(even) = {:?}", even);

    let odd = Some(3).filter(|n| n % 2 == 0);
    println!("Some(3).filter(even) = {:?}", odd);

    // None stays None
    let nothing: MyOption<i32> = None;
    let still_nothing = nothing.filter(|n| n % 2 == 0);
    println!("None.filter(even) = {:?}", still_nothing);

    // Range validation
    let in_range = Some(15).filter(|n| *n >= 10 && *n <= 20);
    println!("Some(15).filter(10..=20) = {:?}", in_range);

    let out_of_range = Some(25).filter(|n| *n >= 10 && *n <= 20);
    println!("Some(25).filter(10..=20) = {:?}", out_of_range);

    // String validation
    let long_name = find_user(1).filter(|n| n.len() > 3);
    println!("Alice filtered (len > 3) = {:?}", long_name);

    let short_name = find_user(2).filter(|n| n.len() > 3);
    println!("Bob filtered (len > 3) = {:?} (too short!)", short_name);
}

fn _09_as_ref() {
    println!("\n--- as_ref (use value without moving) ---");
    let data = Some(String::from("hello world"));

    // Multiple operations on the same Option
    let len = data.as_ref().map(|s| s.len());
    let uppercase = data.as_ref().map(|s| s.to_uppercase());
    let contains = data.as_ref().map(|s| s.contains("world"));

    println!("Length: {:?}", len);
    println!("Uppercase: {:?}", uppercase);
    println!("Contains 'world': {:?}", contains);
    println!("Original still valid: {:?}", data);

    // Without as_ref, data would be moved on first map!
    let maybe_name: MyOption<String> = Some(String::from("Charlie"));
    let len = maybe_name.as_ref().map(|s| s.len());
    println!("Length via as_ref: {:?}", len);
    println!("Original still valid: {:?}", maybe_name);
}

fn _10_take() {
    println!("\n--- take (extract and replace with None) ---");
    let mut slot = Some(String::from("taken"));
    println!("Before take: {:?}", slot);

    let taken = slot.take();
    println!("Taken value: {:?}", taken);
    println!("Slot after take: {:?}", slot);

    // Taking from None returns None
    let mut empty: MyOption<i32> = None;
    let result = empty.take();
    println!("Take from None: {:?}", result);

    // Useful for state machines
    struct Connection {
        state: MyOption<String>,
    }
    let mut conn = Connection {
        state: Some(String::from("connected")),
    };
    let old_state = conn.state.take();
    println!("Old connection state: {:?}", old_state);
    println!("Connection is now: {:?}", conn.state);
}

fn _11_or() {
    println!("\n--- or (provide alternative Option) ---");
    let first: MyOption<i32> = Some(1);
    let second: MyOption<i32> = Some(2);
    println!("Some(1).or(Some(2)) = {:?}", first.or(second));

    let none_first: MyOption<i32> = None;
    let some_second: MyOption<i32> = Some(42);
    println!("None.or(Some(42)) = {:?}", none_first.or(some_second));
}

fn _12_or_else() {
    println!("\n--- or_else (lazy alternative Option) ---");
    let x = Some(1);
    let result = x.or_else(|| {
        println!("  (This is NOT called for Some)");
        Some(2)
    });
    println!("Some(1).or_else(|| Some(2)) = {:?}", result);

    let y: MyOption<i32> = None;
    let result = y.or_else(|| {
        println!("  (This IS called for None)");
        Some(2)
    });
    println!("None.or_else(|| Some(2)) = {:?}", result);
}

fn _13_zip() {
    println!("\n--- zip (combine two Options) ---");
    let a: MyOption<i32> = Some(1);
    let b: MyOption<&str> = Some("hello");
    println!("zip(Some(1), Some(\"hello\")) = {:?}", zip(a, b));

    let c: MyOption<i32> = None;
    let d: MyOption<&str> = Some("world");
    println!("zip(None, Some(\"world\")) = {:?}", zip(c, d));

    let e: MyOption<i32> = Some(42);
    let f: MyOption<&str> = None;
    println!("zip(Some(42), None) = {:?}", zip(e, f));
}

fn _14_flatten() {
    println!("\n--- flatten (remove one level of nesting) ---");
    let nested: MyOption<MyOption<i32>> = Some(Some(42));
    println!("Some(Some(42)).flatten() = {:?}", nested.flatten());

    let nested_none: MyOption<MyOption<i32>> = Some(None);
    println!("Some(None).flatten() = {:?}", nested_none.flatten());

    let outer_none: MyOption<MyOption<i32>> = None;
    println!("None.flatten() = {:?}", outer_none.flatten());
}

fn _15_unzip() {
    println!("\n--- unzip (split tuple Option) ---");
    let tuple: MyOption<(i32, &str)> = Some((1, "hello"));
    let (first, second) = tuple.unzip();
    println!("Some((1, \"hello\")).unzip() = ({:?}, {:?})", first, second);

    let no_tuple: MyOption<(i32, &str)> = None;
    let (a, b) = no_tuple.unzip();
    println!("None.unzip() = ({:?}, {:?})", a, b);
}

fn _16_real_world() {
    println!("\n--- Real-world example: Configuration pipeline ---");
    fn parse_port(s: &str) -> MyOption<u16> {
        match s.parse().ok() {
            std::option::Option::Some(n) => Some(n),
            std::option::Option::None => None,
        }
    }

    fn validate_port(port: u16) -> MyOption<u16> {
        if port > 1024 && port < 65535 {
            Some(port)
        } else {
            None
        }
    }

    let config_value = Some(String::from("8080"));
    let port = config_value
        .and_then(|s| parse_port(&s))
        .and_then(validate_port)
        .unwrap_or(3000);
    println!("Parsed and validated port: {}", port);

    let bad_config = Some(String::from("80"));
    let port = bad_config
        .and_then(|s| parse_port(&s))
        .and_then(validate_port)
        .unwrap_or(3000);
    println!("Invalid port (80 < 1024), using default: {}", port);
}

fn main() {
    println!("=== MyOption Demo ===\n");

    _01_basic_usage();
    _02_is_some_is_none();
    _03_unwrap();
    _04_unwrap_or();
    _05_unwrap_or_else();
    _06_map();
    _07_and_then();
    _08_filter();
    _09_as_ref();
    _10_take();
    _11_or();
    _12_or_else();
    _13_zip();
    _14_flatten();
    _15_unzip();
    _16_real_world();

    println!("\n=== End Demo ===");
}
