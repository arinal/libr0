//! Chapter 3: Box - Exercises
//!
//! Complete the TODO items to practice using Box0 methods.
//! Run with: cargo run --example box

#![allow(unused)]

#[macro_use]
mod common;

use rustlib::r#box::Box0;

// ============================================================================
// Exercises - Replace variables with TODOs with the correct Box0 operations
// ============================================================================

fn _01_new_and_deref() {
    let boxed = Box0::new(42);
    let result = *boxed;

    let boxed_string = Box0::new(String::from("hello"));
    let result2 = boxed_string.len();

    assert_eq!(result, 42);
    assert_eq!(result2, 5);
}

fn _02_deref_mut() {
    let mut boxed = Box0::new(10);
    *boxed = 100;

    let mut boxed_string = Box0::new(String::from("hello"));
    boxed_string.push_str(" world");

    assert_eq!(*boxed, 100);
    assert_eq!(*boxed_string, "hello world");
}

fn _03_into_inner() {
    let boxed = Box0::new(String::from("owned"));
    let result = boxed.into_inner();

    assert_eq!(result, "owned");
    // boxed is no longer valid here
}

fn _04_map() {
    let boxed = Box0::new(5);
    let result: Box0<i32> = boxed.map(|x| x * 2);

    let boxed_str = Box0::new(String::from("hello"));
    let result2: Box0<usize> = boxed_str.map(|s| s.len());

    assert_eq!(*result, 10);
    assert_eq!(*result2, 5);
}

fn _05_clone() {
    let boxed1 = Box0::new(String::from("original"));
    let boxed2 = boxed1.clone();

    assert_eq!(*boxed1, "original");
    assert_eq!(*boxed2, "original");
    // Both boxes own independent copies
}

fn _06_deref_coercion() {
    fn print_len(s: &str) -> usize {
        s.len()
    }

    let boxed_string = Box0::new(String::from("hello"));
    let result = print_len(&boxed_string);

    assert_eq!(result, 5);
}

fn _07_nested_box() {
    let inner = Box0::new(42);
    let outer: Box0<Box0<i32>> = Box0::new(inner);

    let result = **outer;

    assert_eq!(result, 42);
}

fn _08_into_raw_from_raw() {
    let boxed = Box0::new(String::from("raw"));
    let ptr: *mut String = Box0::into_raw(boxed);

    let restored: Box0<String> = unsafe { Box0::from_raw(ptr) };

    assert_eq!(*restored, "raw");
}

// ============================================================================
// Real-world Demo: Recursive Data Structures
// ============================================================================

#[derive(Debug)]
enum List<T> {
    Cons(T, Box0<List<T>>),
    Nil,
}

impl<T> List<T> {
    fn new() -> List<T> {
        List::Nil
    }

    fn prepend(self, value: T) -> List<T> {
        List::Cons(value, Box0::new(self))
    }

    fn len(&self) -> usize {
        match self {
            List::Cons(_, rest) => 1 + rest.len(),
            List::Nil => 0,
        }
    }
}

fn _09_real_world() {
    // Create a list: 1 -> 2 -> 3 -> Nil
    let list: List<i32> = List::new().prepend(3).prepend(2).prepend(1);

    // Without Box0, this wouldn't compile! List would have infinite size.
    // Box0 puts data on the heap and stores only a pointer (8 bytes).
    let list_size = std::mem::size_of::<List<i32>>();

    assert_eq!(list.len(), 3);
    assert_eq!(list_size, 16); // i32 (4 bytes) + pointer (8 bytes) + enum tag (4 bytes padding)
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    run_all![
        "Box0",
        _01_new_and_deref,
        _02_deref_mut,
        _03_into_inner,
        _04_map,
        _05_clone,
        _06_deref_coercion,
        _07_nested_box,
        _08_into_raw_from_raw,
        _09_real_world,
    ];
}
