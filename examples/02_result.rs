//! Chapter 2: Result - Error Handling Done Right
//!
//! Run with: cargo run --example result

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
enum MyResult<T, E> {
    Ok(T),
    Err(E),
}

use MyResult::{Err, Ok};

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

    // Exercise: or
    fn or(self, other: MyResult<T, E>) -> MyResult<T, E> {
        match self {
            Ok(x) => Ok(x),
            Err(_) => other,
        }
    }

    // Exercise: or_else
    fn or_else<F: FnOnce(E) -> MyResult<T, E>>(self, f: F) -> MyResult<T, E> {
        match self {
            Ok(x) => Ok(x),
            Err(e) => f(e),
        }
    }

    // Exercise: and
    fn and<U>(self, other: MyResult<U, E>) -> MyResult<U, E> {
        match self {
            Ok(_) => other,
            Err(e) => Err(e),
        }
    }
}

impl<T, E> MyResult<MyResult<T, E>, E> {
    // Exercise: flatten
    fn flatten(self) -> MyResult<T, E> {
        match self {
            Ok(inner) => inner,
            Err(e) => Err(e),
        }
    }
}

impl<T, E: fmt::Debug> MyResult<T, E> {
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

// ============================================================================
// Demo types
// ============================================================================

#[derive(Debug, Clone)]
enum ParseError {
    Empty,
    InvalidNumber(String),
}

#[derive(Debug, Clone)]
enum ConfigError {
    FileNotFound(String),
    ParseError(ParseError),
    PortOutOfRange(u32),
}

// ============================================================================
// Demo functions
// ============================================================================

fn parse_port(s: &str) -> MyResult<u16, ParseError> {
    let s = s.trim();
    if s.is_empty() {
        return Err(ParseError::Empty);
    }

    match s.parse::<u16>() {
        std::result::Result::Ok(n) => Ok(n),
        std::result::Result::Err(_) => Err(ParseError::InvalidNumber(s.to_string())),
    }
}

fn validate_port(port: u16) -> MyResult<u16, ConfigError> {
    if port < 1024 {
        Err(ConfigError::PortOutOfRange(port as u32))
    } else {
        Ok(port)
    }
}

fn read_config(content: &str) -> MyResult<u16, ConfigError> {
    parse_port(content)
        .map_err(ConfigError::ParseError)
        .and_then(validate_port)
}

fn _01_basic_usage() {
    println!("--- Basic Usage ---");
    match parse_port("8080") {
        Ok(port) => println!("Parsed port: {}", port),
        Err(e) => println!("Parse error: {:?}", e),
    }
}

fn _02_is_ok_is_err() {
    println!("\n--- is_ok / is_err ---");
    let good: MyResult<u16, ParseError> = parse_port("3000");
    let bad: MyResult<u16, ParseError> = parse_port("not a number");
    println!("parse_port(\"3000\").is_ok() = {}", good.is_ok());
    println!("parse_port(\"not a number\").is_err() = {}", bad.is_err());
}

fn _03_expect() {
    println!("\n--- expect ---");
    let port = parse_port("8080").expect("Port must be valid");
    println!("parse_port(\"8080\").expect(...) = {}", port);
    // This would panic with custom message:
    // parse_port("bad").expect("Port must be valid");
    // Panics: "Port must be valid: InvalidNumber(\"bad\")"
}

fn _04_unwrap_or() {
    println!("\n--- unwrap_or ---");
    let port = parse_port("invalid").unwrap_or(8080);
    println!("parse_port(\"invalid\").unwrap_or(8080) = {}", port);
}

fn _05_unwrap_or_else() {
    println!("\n--- unwrap_or_else ---");
    let port = parse_port("").unwrap_or_else(|e| {
        println!("  Warning: {:?}, using default", e);
        8080
    });
    println!("Final port: {}", port);
}

fn _06_map() {
    println!("\n--- map ---");
    let doubled = parse_port("100").map(|p| p * 2);
    println!("parse_port(\"100\").map(|p| p * 2) = {:?}", doubled);

    let still_err = parse_port("bad").map(|p| p * 2);
    println!("parse_port(\"bad\").map(|p| p * 2) = {:?}", still_err);
}

fn _07_map_err() {
    println!("\n--- map_err ---");
    let converted: MyResult<u16, String> =
        parse_port("bad").map_err(|e| format!("Config error: {:?}", e));
    println!("Error converted to String: {:?}", converted);
}

fn _08_and_then() {
    println!("\n--- and_then (chaining) ---");
    let result = read_config("8080");
    println!("read_config(\"8080\") = {:?}", result);

    let result = read_config("80"); // too low
    println!("read_config(\"80\") = {:?}", result);

    let result = read_config("not a port");
    println!("read_config(\"not a port\") = {:?}", result);
}

fn _09_as_ref() {
    println!("\n--- as_ref (use without moving) ---");
    let result: MyResult<String, ParseError> = Ok(String::from("test"));

    // Multiple operations on the same Result
    let len = result.as_ref().map(|s| s.len());
    let uppercase = result.as_ref().map(|s| s.to_uppercase());
    let is_empty = result.as_ref().map(|s| s.is_empty());

    println!("Length: {:?}", len);
    println!("Uppercase: {:?}", uppercase);
    println!("Is empty: {:?}", is_empty);
    println!("Original still valid: {:?}", result);

    // Without as_ref, first map would consume the result!
}

fn _10_ok() {
    println!("\n--- ok() ---");
    let maybe_port: Option<u16> = parse_port("3000").ok();
    println!("parse_port(\"3000\").ok() = {:?}", maybe_port);

    let no_port: Option<u16> = parse_port("bad").ok();
    println!("parse_port(\"bad\").ok() = {:?}", no_port);
}

fn _11_err() {
    println!("\n--- err() ---");
    let the_error: Option<ParseError> = parse_port("xyz").err();
    println!("parse_port(\"xyz\").err() = {:?}", the_error);
}

fn _12_or() {
    println!("\n--- or ---");
    let primary: MyResult<u16, ParseError> = parse_port("bad");
    let fallback: MyResult<u16, ParseError> = Ok(8080);
    println!("parse_port(\"bad\").or(Ok(8080)) = {:?}", primary.or(fallback));
}

fn _13_and() {
    println!("\n--- and ---");
    let first: MyResult<u16, ParseError> = parse_port("3000");
    let second: MyResult<&str, ParseError> = Ok("valid");
    println!(
        "parse_port(\"3000\").and(Ok(\"valid\")) = {:?}",
        first.and(second)
    );

    let first: MyResult<u16, ParseError> = parse_port("bad");
    let second: MyResult<&str, ParseError> = Ok("valid");
    println!(
        "parse_port(\"bad\").and(Ok(\"valid\")) = {:?}",
        first.and(second)
    );
}

fn _14_flatten() {
    println!("\n--- flatten ---");
    let nested: MyResult<MyResult<u16, ParseError>, ParseError> = Ok(Ok(42));
    println!("Ok(Ok(42)).flatten() = {:?}", nested.flatten());

    let nested_err: MyResult<MyResult<u16, ParseError>, ParseError> = Ok(Err(ParseError::Empty));
    println!("Ok(Err(Empty)).flatten() = {:?}", nested_err.flatten());
}

fn _15_config_pipeline() {
    println!("\n--- Practical: Config Pipeline ---");
    let configs = ["8080", "443", "invalid", "", "80"];
    for input in configs {
        let result = read_config(input);
        match result {
            Ok(port) => println!("  '{}' -> Valid port: {}", input, port),
            Err(e) => println!("  '{}' -> Error: {:?}", input, e),
        }
    }
}

fn _16_or_else() {
    println!("\n--- or_else (lazy alternative) ---");
    let primary: MyResult<u16, ParseError> = parse_port("8080");
    let result = primary.or_else(|_e| {
        println!("  (This is NOT called for Ok)");
        Ok(3000)
    });
    println!("parse_port(\"8080\").or_else(...) = {:?}", result);

    let failed: MyResult<u16, ParseError> = parse_port("bad");
    let result = failed.or_else(|e| {
        println!("  (This IS called for Err, error was: {:?})", e);
        Ok(3000)
    });
    println!("parse_port(\"bad\").or_else(...) = {:?}", result);

    // Chaining fallbacks
    let result = parse_port("invalid")
        .or_else(|_| parse_port("8080"))
        .or_else(|_| Ok(3000));
    println!("Chained fallbacks result: {:?}", result);
}

fn main() {
    println!("=== MyResult Demo ===\n");

    _01_basic_usage();
    _02_is_ok_is_err();
    _03_expect();
    _04_unwrap_or();
    _05_unwrap_or_else();
    _06_map();
    _07_map_err();
    _08_and_then();
    _09_as_ref();
    _10_ok();
    _11_err();
    _12_or();
    _13_and();
    _14_flatten();
    _15_config_pipeline();
    _16_or_else();

    println!("\n=== End Demo ===");
}
