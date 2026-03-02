//! Chapter 2: Result - Exercises
//!
//! Complete the TODO items to practice using Result0 methods.
//! Run with: cargo run --example 02_result

#![allow(unused)]

#[macro_use]
mod common;

use rustlib::result::{Err, Ok, Result0};

// ============================================================================
// Exercises - Replace variables with TODOs with the correct Result0 method calls
// ============================================================================

fn _01_is_ok_is_err() {
    let value: Result0<i32, &str> = Ok(42);
    let result = value.is_ok();

    let error: Result0<i32, &str> = Err("something went wrong");
    let result2 = error.is_err();

    assert!(result);
    assert!(result2);
}

fn _02_ok_err() {
    let value: Result0<i32, &str> = Ok(42);
    let result: Option<i32> = value.ok();

    let error: Result0<i32, &str> = Err("not found");
    let result2: Option<&str> = error.err();

    assert_eq!(result, Some(42));
    assert_eq!(result2, Some("not found"));
}

fn _03_unwrap_or() {
    let port: Result0<u16, &str> = Err("config missing");
    let result = port.unwrap_or(8080);

    let timeout: Result0<u32, &str> = Ok(30);
    let result2 = timeout.unwrap_or(60);

    assert_eq!(result, 8080);
    assert_eq!(result2, 30);
}

fn _04_unwrap_or_else() {
    fn compute_default(err: &str) -> i32 {
        err.len() as i32 * 10
    }

    let cache: Result0<i32, &str> = Err("miss");
    let result = cache.unwrap_or_else(compute_default);

    let cache2: Result0<i32, &str> = Ok(100);
    let result2 = cache2.unwrap_or_else(compute_default);

    assert_eq!(result, 40); // "miss".len() * 10
    assert_eq!(result2, 100);
}

fn _05_map() {
    let value: Result0<i32, &str> = Ok(10);
    let result: Result0<i32, &str> = value.map(|x| x * 2);

    let to_string: Result0<String, &str> = Ok(10).map(|x| x.to_string());

    let error: Result0<i32, &str> = Err("error");
    let result3: Result0<i32, &str> = error.map(|x| x * 2);

    assert_eq!(result, Ok(20));
    assert_eq!(to_string, Ok(String::from("10")));
    assert_eq!(result3, Err("error"));
}

fn _06_map_err() {
    let value: Result0<i32, &str> = Ok(42);
    let result: Result0<i32, usize> = value.map_err(|e| e.len());

    let error: Result0<i32, &str> = Err("not found");
    let result2: Result0<i32, usize> = error.map_err(|e| e.len());

    assert_eq!(result, Ok(42));
    assert_eq!(result2, Err(9));
}

fn _07_and_then() {
    #[derive(Debug, PartialEq)]
    struct User {
        name: String,
        age: i32,
    }

    fn validate_name(name: String) -> Result0<String, String> {
        if name.is_empty() {
            Err("name cannot be empty".to_string())
        } else {
            Ok(name)
        }
    }

    fn validate_age(age: i32) -> Result0<i32, String> {
        if age <= 0 {
            Err("age must be positive".to_string())
        } else {
            Ok(age)
        }
    }

    fn create_user(name: String, age: i32) -> Result0<User, String> {
        validate_name(name).and_then(|n| validate_age(age).map(|a| User { name: n, age: a }))
    }

    // Valid user
    let valid: Result0<User, String> = create_user(String::from("Alice"), 30);

    // Invalid name
    let invalid_name: Result0<User, String> = create_user(String::from(""), 25);

    // Invalid age
    let invalid_age: Result0<User, String> = create_user(String::from("Bob"), -5);

    // Monadic short-circuiting: even though both name and age are invalid,
    // only the first error is returned because `and_then` stops at the first failure.
    // This prevents unnecessary computation and is why Result is called a "monad".
    let invalid_age_and_name: Result0<User, String> = create_user(String::new(), -5);

    assert_eq!(
        valid,
        Ok(User {
            name: String::from("Alice"),
            age: 30
        })
    );
    assert_eq!(invalid_name, Err("name cannot be empty".to_string()));
    assert_eq!(invalid_age, Err("age must be positive".to_string()));
    assert_eq!(
        invalid_age_and_name,
        Err("name cannot be empty".to_string())
    );
}

fn _08_as_ref() {
    let message: Result0<String, &str> = Ok(String::from("Hello!"));
    let length: Result0<usize, &&str> = message.as_ref().map(|s| s.len());

    let contains: Result0<bool, &&str> = message.as_ref().map(|s| s.contains('H'));

    assert_eq!(length, Ok(6));
    assert_eq!(contains, Ok(true));
    assert_eq!(message, Ok(String::from("Hello!"))); // message is still available here,
                                                     // the value is not moved out
}

fn _09_or() {
    let primary: Result0<i32, &str> = Err("primary failed");
    let fallback: Result0<i32, &str> = Ok(42);
    let result: Result0<i32, &str> = primary.or(fallback);

    let primary2: Result0<i32, &str> = Ok(10);
    let fallback2: Result0<i32, &str> = Ok(42);
    let result2: Result0<i32, &str> = primary2.or(fallback2);

    assert_eq!(result, Ok(42));
    assert_eq!(result2, Ok(10));
}

fn _10_or_else() {
    fn recover(err: &str) -> Result0<i32, &str> {
        Ok(err.len() as i32)
    }

    let cache: Result0<i32, &str> = Ok(100);
    let result: Result0<i32, &str> = cache.or_else(recover);

    let cache2: Result0<i32, &str> = Err("failed");
    let result2: Result0<i32, &str> = cache2.or_else(recover);

    assert_eq!(result, Ok(100));
    assert_eq!(result2, Ok(6));
}

fn _11_and() {
    let first: Result0<i32, &str> = Ok(1);
    let second: Result0<&str, &str> = Ok("hello");
    let result: Result0<&str, &str> = first.and(second);

    let error: Result0<i32, &str> = Err("first failed");
    let second2: Result0<&str, &str> = Ok("world");
    let result2: Result0<&str, &str> = error.and(second2);

    assert_eq!(result, Ok("hello"));
    assert_eq!(result2, Err("first failed"));
}

fn _12_flatten() {
    let not_nested: Result0<i32, &str> = Ok(42);
    // try that a non-nested result don't have `flatten` method
    // let _ = not_nested.flatten(); // This should not compile

    let nested: Result0<Result0<i32, &str>, &str> = Ok(Ok(42));
    let result: Result0<i32, &str> = nested.flatten();

    let nested2: Result0<Result0<i32, &str>, &str> = Ok(Err("inner error"));
    let result2: Result0<i32, &str> = nested2.flatten();

    let nested3: Result0<Result0<i32, &str>, &str> = Err("outer error");
    let result3: Result0<i32, &str> = nested3.flatten();

    assert_eq!(result, Ok(42));
    assert_eq!(result2, Err("inner error"));
    assert_eq!(result3, Err("outer error"));
}

// ============================================================================
// Real-world Demo: File Config Parser
// ============================================================================

#[derive(Debug, PartialEq)]
struct Config {
    host: String,
    port: u16,
}

fn parse_port(s: &str) -> Result0<u16, String> {
    s.parse::<u16>()
        .map(Ok)
        .unwrap_or_else(|_| Err(format!("invalid port: {}", s)))
}

fn load_config(host: &str, port_str: &str) -> Result0<Config, String> {
    if host.is_empty() {
        return Err(String::from("host cannot be empty"));
    }

    parse_port(port_str).map(|port| Config {
        host: host.to_string(),
        port,
    })
}

fn _13_real_world() {
    // Valid config
    let result: Result0<Config, String> = load_config("localhost", "8080");

    // Invalid port
    let result2 = load_config("localhost", "abc");
    let error_msg = result2.err().unwrap();

    // Empty host with default
    let result3 = load_config("", "3000");
    let host = result3.map(|c| c.host).unwrap_or(String::from("0.0.0.0"));

    // Chain operations: load config, map to port, then add 100
    let result4: Result0<u16, String> = load_config("localhost", "8080").map(|c| c.port + 100);

    assert_eq!(
        result,
        Ok(Config {
            host: String::from("localhost"),
            port: 8080
        })
    );
    assert_eq!(error_msg, String::from("invalid port: abc"));
    assert_eq!(host, String::from("0.0.0.0"));
    assert_eq!(result4, Ok(8180));
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    run_all![
        "Result0",
        _01_is_ok_is_err,
        _02_ok_err,
        _03_unwrap_or,
        _04_unwrap_or_else,
        _05_map,
        _06_map_err,
        _07_and_then,
        _08_as_ref,
        _09_or,
        _10_or_else,
        _11_and,
        _12_flatten,
        _13_real_world,
    ];
}
