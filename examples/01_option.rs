//! Chapter 1: Option - Exercises
//!
//! Complete the TODO items to practice using Option0 methods.
//! Run with: cargo run --example 01_option

#![allow(unused)]

#[macro_use]
mod common;

use rustlib::option::{zip, Option0, None, Some};

// ============================================================================
// Exercises - Replace variables with TODOs with the correct Option0 method calls
// ============================================================================

fn _01_is_some_is_none() {
    let value: Option0<i32> = Some(42);
    let result = false; // TODO: check if value is some, e.g. value.is_some()

    let empty: Option0<i32> = None;
    let result2 = false; // TODO: check if empty is none

    assert!(result);
    assert!(result2);
}

fn _02_unwrap_or() {
    let port: Option0<u16> = None;
    let result = 0; // TODO: get port or default to 8080

    let timeout: Option0<u32> = Some(30);
    let result2 = 0; // TODO: get timeout or default to 60

    assert_eq!(result, 8080);
    assert_eq!(result2, 30);
}

fn _03_unwrap_or_else() {
    fn expensive_computation() -> i32 {
        42
    }

    let cache: Option0<i32> = None;
    let result = 0; // TODO: get cache or compute expensive value using expensive_computation

    let cache2: Option0<i32> = Some(100);
    let result2 = 0; // TODO: get cache or compute (should not call function)

    assert_eq!(result, 42);
    assert_eq!(result2, 100);
}

fn _04_map() {
    let name: Option0<&str> = Some("alice");
    let result: Option0<&str> = None; // TODO: convert name to uppercase
    let length: Option0<usize> = None; // TODO: get length of name

    let empty: Option0<&str> = None;
    let result3: Option0<&str> = Some(""); // TODO: map empty to uppercase

    assert_eq!(result, Some("ALICE"));
    assert_eq!(length, Some(5));
    assert_eq!(result3, None);
}

fn _05_and_then() {
    fn safe_divide(a: i32, b: i32) -> Option0<i32> {
        if b == 0 {
            None
        } else {
            Some(a / b)
        }
    }

    let value: Option0<i32> = Some(20);
    let result: Option0<i32> = None; // TODO: divide by 4, then divide by 5 using and_then

    let value2: Option0<i32> = Some(10);
    let result2: Option0<i32> = Some(0); // TODO: divide by 0 using and_then

    let a = Some(1);
    let b = Some(2);
    let c = Some(3);
    let result3: Option0<i32> = None; // TODO: sum a, b, c using and_then

    assert_eq!(result, Some(1));
    assert_eq!(result2, None);
}

fn _06_filter() {
    let value = Some(15);
    let result: Option0<i32> = None; // TODO: keep value if >= 10

    let value2 = Some(5);
    let result2: Option0<i32> = Some(0); // TODO: keep value if >= 10

    let value3: Option0<i32> = None;
    let result3: Option0<i32> = Some(0); // TODO: filter none

    assert_eq!(result, Some(15));
    assert_eq!(result2, None);
    assert_eq!(result3, None);
}

fn _07_as_ref() {
    let message = Some(String::from("Hello, world!"));
    let length: Option0<usize> = None; // TODO: get length without moving message
    let contains: Option0<bool> = None; // TODO: check if contains 'world' without moving message

    assert_eq!(length, Some(11));
    assert_eq!(contains, Some(true));
    assert_eq!(message, Some(String::from("Hello, world!")));
}

fn _08_take() {
    let mut slot = Some(String::from("data"));
    let result: Option0<String> = None; // TODO: take value from slot

    assert_eq!(result, Some(String::from("data")));
    assert_eq!(slot, None);
}

fn _09_or() {
    let primary: Option0<i32> = None;
    let fallback: Option0<i32> = Some(42);
    let result: Option0<i32> = None; // TODO: use primary or fallback

    let primary2: Option0<i32> = Some(10);
    let fallback2: Option0<i32> = Some(42);
    let result2: Option0<i32> = None; // TODO: use primary or fallback

    assert_eq!(result, Some(42));
    assert_eq!(result2, Some(10));
}

fn _10_or_else() {
    fn compute() -> Option0<i32> {
        Some(42)
    }

    let cache: Option0<i32> = Some(100);
    let result: Option0<i32> = None; // TODO: use cache or compute

    let cache2: Option0<i32> = None;
    let result2: Option0<i32> = None; // TODO: compute if cache is none using or_else

    assert_eq!(result, Some(100));
    assert_eq!(result2, Some(42));
}

fn _11_zip() {
    let a: Option0<i32> = Some(1);
    let b: Option0<&str> = Some("hello");
    let result: Option0<(i32, &str)> = None; // TODO: zip a and b

    let a2: Option0<i32> = None;
    let b2: Option0<&str> = Some("world");
    let result2: Option0<(i32, &str)> = Some((0, "")); // TODO: zip none and some

    assert_eq!(result, Some((1, "hello")));
    assert_eq!(result2, None);
}

fn _12_flatten() {
    let not_nested: Option0<i32> = Some(42);
    // try that a non-nested option don't have `flatten` method
    // let _ = not_nested.flatten(); // This should not compile

    let nested: Option0<Option0<i32>> = Some(Some(42));
    let result: Option0<i32> = None; // TODO: flatten nested

    let nested2: Option0<Option0<i32>> = Some(None);
    let result2: Option0<i32> = Some(0); // TODO: flatten some(none)

    assert_eq!(result, Some(42));
    assert_eq!(result2, None);
}

fn _13_unzip() {
    let pair: Option0<(i32, &str)> = Some((42, "answer"));
    let (num, text) = (None, None); // TODO: unzip pair

    let pair2: Option0<(i32, &str)> = None;
    let (num2, text2) = (Some(0), Some("")); // TODO: unzip none

    assert_eq!(num, Some(42));
    assert_eq!(text, Some("answer"));
    assert_eq!(num2, None);
    assert_eq!(text2, None);
}

// ============================================================================
// Real-world Demo: User Profile Lookup
// ============================================================================

#[derive(Debug, PartialEq, Clone)]
struct User {
    id: u32,
    name: String,
    email: Option0<String>,
}

fn find_user(id: u32) -> Option0<User> {
    match id {
        1 => Some(User {
            id: 1,
            name: String::from("Alice"),
            email: Some(String::from("alice@example.com")),
        }),
        2 => Some(User {
            id: 2,
            name: String::from("Bob"),
            email: None,
        }),
        _ => None,
    }
}

fn _14_real_world() {
    // Find user and extract email
    let result: Option0<String> = None; // TODO: find user 1 and get their email

    // User with no email - provide default
    let user = find_user(2);
    let result2 = String::new(); // TODO: get user 2's email or use 'noreply@example.com'

    // Check if user exists
    let result3 = false; // TODO: check if user 999 exists using is_none

    // Chain operations: find user, then check if email contains '@'
    let result4: Option0<bool> = None; // TODO: find user 1, get email, check if contains '@'

    assert_eq!(result, Some(String::from("alice@example.com")));
    assert_eq!(result2, String::from("noreply@example.com"));
    assert!(result3);
    assert_eq!(result4, Some(true));
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    run_all!["Option0",
        _01_is_some_is_none,
        _02_unwrap_or,
        _03_unwrap_or_else,
        _04_map,
        _05_and_then,
        _06_filter,
        _07_as_ref,
        _08_take,
        _09_or,
        _10_or_else,
        _11_zip,
        _12_flatten,
        _13_unzip,
        _14_real_world,
    ];
}
