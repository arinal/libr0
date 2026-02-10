//! Chapter 2: Result - Exercises
//!
//! Complete the TODO items to practice using MyResult methods.
//! Run with: cargo run --example 02_result

#![allow(unused)]

#[macro_use]
mod common;

use rustlib::result::{Err, MyResult, Ok};

// ============================================================================
// Exercises - Replace variables with TODOs with the correct MyResult method calls
// ============================================================================

fn _01_is_ok_is_err() {
    let value: MyResult<i32, &str> = Ok(42);
    let result = false; // TODO: check if value is ok, e.g. value.is_ok()

    let error: MyResult<i32, &str> = Err("something went wrong");
    let result2 = false; // TODO: check if error is err

    assert!(result);
    assert!(result2);
}

fn _02_ok_err() {
    let value: MyResult<i32, &str> = Ok(42);
    let result: Option<i32> = None; // TODO: convert to Option using ok()

    let error: MyResult<i32, &str> = Err("not found");
    let result2: Option<&str> = None; // TODO: extract error using err()

    assert_eq!(result, Some(42));
    assert_eq!(result2, Some("not found"));
}

fn _03_unwrap_or() {
    let port: MyResult<u16, &str> = Err("config missing");
    let result = 0; // TODO: get port or default to 8080

    let timeout: MyResult<u32, &str> = Ok(30);
    let result2 = 0; // TODO: get timeout or default to 60

    assert_eq!(result, 8080);
    assert_eq!(result2, 30);
}

fn _04_unwrap_or_else() {
    fn compute_default(err: &str) -> i32 {
        err.len() as i32 * 10
    }

    let cache: MyResult<i32, &str> = Err("miss");
    let result = 0; // TODO: get cache or compute from error using compute_default

    let cache2: MyResult<i32, &str> = Ok(100);
    let result2 = 0; // TODO: get cache or compute (should not call function)

    assert_eq!(result, 40); // "miss".len() * 10
    assert_eq!(result2, 100);
}

fn _05_map() {
    let value: MyResult<i32, &str> = Ok(10);
    let result: MyResult<i32, &str> = Ok(0); // TODO: multiply by 2

    let to_string: MyResult<String, &str> = Ok(String::new()); // TODO: map Ok(10) to string "10"

    let error: MyResult<i32, &str> = Err("error");
    let result3: MyResult<i32, &str> = Ok(0); // TODO: map error to multiply by 2

    assert_eq!(result, Ok(20));
    assert_eq!(to_string, Ok(String::from("10")));
    assert_eq!(result3, Err("error"));
}

fn _06_map_err() {
    let value: MyResult<i32, &str> = Ok(42);
    let result: MyResult<i32, usize> = Err(0); // TODO: map error to its length

    let error: MyResult<i32, &str> = Err("not found");
    let result2: MyResult<i32, usize> = Err(0); // TODO: map error to its length

    assert_eq!(result, Ok(42));
    assert_eq!(result2, Err(9));
}

fn _07_and_then() {
    #[derive(Debug, PartialEq)]
    struct User {
        name: String,
        age: i32,
    }

    fn validate_name(name: String) -> MyResult<String, String> {
        if name.is_empty() {
            Err("name cannot be empty".to_string())
        } else {
            Ok(name)
        }
    }

    fn validate_age(age: i32) -> MyResult<i32, String> {
        if age <= 0 {
            Err("age must be positive".to_string())
        } else {
            Ok(age)
        }
    }

    fn create_user(name: String, age: i32) -> MyResult<User, String> {
        todo!("use validate_name and validate_age with and_then to create a User");
        todo!("Alternatively, you can use the ? operator to simplify the code")
    }

    // Valid user
    let valid: MyResult<User, &str> = Err(""); // TODO: create user with "Alice" and 30

    // Invalid name
    let invalid_name: MyResult<User, String> = Ok(User {
        name: String::new(),
        age: 0,
    }); // TODO: create user with "" and 25

    // Invalid age
    let invalid_age: MyResult<User, String> = Ok(User {
        name: String::new(),
        age: 0,
    }); // TODO: create user with "Bob" and -5

    // Monadic short-circuiting: even though both name and age are invalid,
    // only the first error is returned because `and_then` stops at the first failure.
    // This prevents unnecessary computation and is why Result is called a "monad".
    let invalid_age_and_name: MyResult<User, String> = create_user(String::new(), -5);

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
    let message: MyResult<String, &str> = Ok(String::from("Hello!"));
    let length: MyResult<usize, &&str> = Ok(0); // TODO: get length without moving message

    let contains: MyResult<bool, &&str> = Ok(false); // TODO: check if contains 'H' without moving message

    assert_eq!(length, Ok(6));
    assert_eq!(contains, Ok(true));
    assert_eq!(message, Ok(String::from("Hello!"))); // message is still available here,
                                                     // the value is not moved out
}

fn _09_or() {
    let primary: MyResult<i32, &str> = Err("primary failed");
    let fallback: MyResult<i32, &str> = Ok(42);
    let result: MyResult<i32, &str> = Err(""); // TODO: use primary or fallback

    let primary2: MyResult<i32, &str> = Ok(10);
    let fallback2: MyResult<i32, &str> = Ok(42);
    let result2: MyResult<i32, &str> = Err(""); // TODO: use primary or fallback

    assert_eq!(result, Ok(42));
    assert_eq!(result2, Ok(10));
}

fn _10_or_else() {
    fn recover(err: &str) -> MyResult<i32, &str> {
        Ok(err.len() as i32)
    }

    let cache: MyResult<i32, &str> = Ok(100);
    let result: MyResult<i32, &str> = Err(""); // TODO: use cache or recover

    let cache2: MyResult<i32, &str> = Err("failed");
    let result2: MyResult<i32, &str> = Err(""); // TODO: recover using or_else

    assert_eq!(result, Ok(100));
    assert_eq!(result2, Ok(6));
}

fn _11_and() {
    let first: MyResult<i32, &str> = Ok(1);
    let second: MyResult<&str, &str> = Ok("hello");
    let result: MyResult<&str, &str> = Err(""); // TODO: combine using and

    let error: MyResult<i32, &str> = Err("first failed");
    let second2: MyResult<&str, &str> = Ok("world");
    let result2: MyResult<&str, &str> = Ok(""); // TODO: combine error and second2 using and

    assert_eq!(result, Ok("hello"));
    assert_eq!(result2, Err("first failed"));
}

fn _12_flatten() {
    let not_nested: MyResult<i32, &str> = Ok(42);
    // try that a non-nested result don't have `flatten` method
    // let _ = not_nested.flatten(); // This should not compile

    let nested: MyResult<MyResult<i32, &str>, &str> = Ok(Ok(42));
    let result: MyResult<i32, &str> = Err(""); // TODO: flatten nested

    let nested2: MyResult<MyResult<i32, &str>, &str> = Ok(Err("inner error"));
    let result2: MyResult<i32, &str> = Ok(0); // TODO: flatten ok(err)

    let nested3: MyResult<MyResult<i32, &str>, &str> = Err("outer error");
    let result3: MyResult<i32, &str> = Ok(0); // TODO: flatten err

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

fn parse_port(s: &str) -> MyResult<u16, String> {
    s.parse::<u16>()
        .map(Ok)
        .unwrap_or_else(|_| Err(format!("invalid port: {}", s)))
}

fn load_config(host: &str, port_str: &str) -> MyResult<Config, String> {
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
    let result: MyResult<Config, String> = Err(String::new()); // TODO: load config with "localhost" and "8080"

    // Invalid port
    let result2 = load_config("localhost", "abc");
    let error_msg = String::new(); // TODO: extract error message using unwrap_err or err().unwrap()

    // Empty host with default
    let result3 = load_config("", "3000");
    let host = String::new(); // TODO: get host or default to "0.0.0.0" using unwrap_or (you need to extract host field first)

    // Chain operations: load config, map to port, then add 100
    let result4: MyResult<u16, String> = Err(String::new()); // TODO: load config, map to get port + 100

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
        "MyResult",
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
