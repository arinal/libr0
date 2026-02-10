//! Chapter 3: Box - Exercises
//!
//! Complete the TODO items to practice using MyBox methods.
//! Run with: cargo run --example box

#![allow(unused)]

#[macro_use]
mod common;

use rustlib::r#box::MyBox;

// ============================================================================
// Exercises - Replace variables with TODOs with the correct MyBox operations
// ============================================================================

fn _01_new_and_deref() {
    let boxed = MyBox::new(42);
    let result = 0; // TODO: dereference boxed to get the value

    let boxed_string = MyBox::new(String::from("hello"));
    let result2 = 0; // TODO: get the length of the string inside boxed_string

    assert_eq!(result, 42);
    assert_eq!(result2, 5);
}

fn _02_deref_mut() {
    let mut boxed = MyBox::new(10);
    // TODO: use dereference mutation to change the value to 100

    let mut boxed_string = MyBox::new(String::from("hello"));
    // TODO: use push_str to add " world" to the string

    assert_eq!(*boxed, 100);
    assert_eq!(*boxed_string, "hello world");
}

fn _03_into_inner() {
    let boxed = MyBox::new(String::from("owned"));
    let result = String::new(); // TODO: extract the String from boxed using into_inner

    assert_eq!(result, "owned");
    // boxed is no longer valid here
}

fn _04_map() {
    let boxed = MyBox::new(5);
    let result: MyBox<i32> = MyBox::new(0); // TODO: map boxed to multiply by 2

    let boxed_str = MyBox::new(String::from("hello"));
    let result2: MyBox<usize> = MyBox::new(0); // TODO: map to get length

    assert_eq!(*result, 10);
    assert_eq!(*result2, 5);
}

fn _05_clone() {
    let boxed1 = MyBox::new(String::from("original"));
    let boxed2 = MyBox::new(String::new()); // TODO: clone boxed1

    assert_eq!(*boxed1, "original");
    assert_eq!(*boxed2, "original");
    // Both boxes own independent copies
}

fn _06_deref_coercion() {
    fn print_len(s: &str) -> usize {
        s.len()
    }

    let boxed_string = MyBox::new(String::from("hello"));
    let result = 0; // TODO: call print_len with &boxed_string (deref coercion!)

    assert_eq!(result, 5);
}

fn _07_nested_box() {
    let inner = MyBox::new(42);
    let outer: MyBox<MyBox<i32>> = MyBox::new(MyBox::new(0)); // TODO: wrap inner in another MyBox

    let result = 0; // TODO: dereference twice to get the value

    assert_eq!(result, 42);
}

fn _08_into_raw_from_raw() {
    let boxed = MyBox::new(String::from("raw"));
    let ptr: *mut String = std::ptr::null_mut(); // TODO: convert boxed to raw pointer using into_raw

    let restored: MyBox<String> = MyBox::new(String::new()); // TODO: restore from ptr using from_raw (unsafe!)

    assert_eq!(*restored, "raw");
}

// ============================================================================
// Real-world Demo: Recursive Data Structures
// ============================================================================

#[derive(Debug)]
enum List<T> {
    Cons(T, MyBox<List<T>>),
    Nil,
}

impl<T> List<T> {
    fn new() -> List<T> {
        List::Nil
    }

    fn prepend(self, value: T) -> List<T> {
        List::Cons(value, MyBox::new(self))
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
    let list: List<i32> = List::Nil; // TODO: create list with values 3, 2, 1 using prepend

    // Without MyBox, this wouldn't compile! List would have infinite size.
    // MyBox puts data on the heap and stores only a pointer (8 bytes).
    let list_size = std::mem::size_of::<List<i32>>();

    assert_eq!(list.len(), 3);
    assert_eq!(list_size, 16); // i32 (4 bytes) + pointer (8 bytes) + enum tag (4 bytes padding)
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    run_all![
        "MyBox",
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