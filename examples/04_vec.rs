//! Chapter 4: Vec - Exercises
//!
//! Complete the TODO items to practice using MyVec methods.
//! Run with: cargo run --example vec

#![allow(unused)]

#[macro_use]
mod common;

#[macro_use]
extern crate rustlib;

use rustlib::vec::MyVec;

// ============================================================================
// Exercises - Replace variables with TODOs with the correct MyVec operations
// ============================================================================

fn _01_new_and_push() {
    let mut vec: MyVec<i32> = MyVec::new();
    // TODO: push 10, 20, 30 to vec

    let result = 0; // TODO: get the length of vec
    let value = 0; // TODO: get vec[1]

    assert_eq!(result, 3);
    assert_eq!(value, 20);
}

fn _02_with_capacity() {
    let mut vec: MyVec<i32> = MyVec::new(); // TODO: create vec with capacity 10

    let cap1 = 0; // TODO: get capacity before pushing

    vec.push(1);
    vec.push(2);

    let cap2 = 0; // TODO: get capacity after pushing

    assert_eq!(cap1, 10);
    assert_eq!(cap2, 10); // No reallocation needed
}

fn _03_pop() {
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    let result1: Option<i32> = None; // TODO: pop from vec
    let result2: Option<i32> = None; // TODO: pop again
    let len = 0; // TODO: get length after pops

    assert_eq!(result1, Some(3));
    assert_eq!(result2, Some(2));
    assert_eq!(len, 1);
}

fn _04_indexing() {
    let mut vec = MyVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    // TODO: change vec[1] to 99

    let result = 0; // TODO: get vec[1]

    assert_eq!(result, 99);
}

fn _05_insert() {
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(3);

    // TODO: insert 2 at index 1

    assert_eq!(vec[0], 1);
    assert_eq!(vec[1], 2);
    assert_eq!(vec[2], 3);
}

fn _06_remove() {
    let mut vec = MyVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);
    vec.push(40);

    let removed = 0; // TODO: remove element at index 1

    assert_eq!(removed, 20);
    assert_eq!(vec.len(), 3);
    assert_eq!(vec[1], 30); // Elements shifted left
}

fn _07_clear() {
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    let cap_before = vec.capacity();
    // TODO: clear the vec
    let len_after = 0; // TODO: get length after clear

    assert_eq!(len_after, 0);
    assert_eq!(vec.capacity(), cap_before); // Capacity unchanged
}

fn _08_shrink_to_fit() {
    let mut vec = MyVec::with_capacity(10);
    vec.push(1);
    vec.push(2);

    // TODO: shrink capacity to match length

    assert_eq!(vec.capacity(), 2);
    assert_eq!(vec.len(), 2);
}

fn _09_as_slice() {
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    let slice: &[i32] = &[]; // TODO: get vec as slice using as_slice()
    let sum = 0; // TODO: sum all elements in slice using iter()

    assert_eq!(slice.len(), 3);
    assert_eq!(sum, 6);
}

fn _10_deref_coercion() {
    let mut vec = MyVec::new();
    vec.push(3);
    vec.push(1);
    vec.push(2);

    // TODO: sort vec using the sort() method from [T]

    assert_eq!(vec[0], 1);
    assert_eq!(vec[1], 2);
    assert_eq!(vec[2], 3);
}

fn _11_clone() {
    let mut vec1 = MyVec::new();
    vec1.push(String::from("hello"));
    vec1.push(String::from("world"));

    let vec2: MyVec<String> = MyVec::new(); // TODO: clone vec1

    vec1[0] = String::from("changed");

    assert_eq!(vec1[0], "changed");
    assert_eq!(vec2[0], "hello"); // Independent copy
}

fn _12_into_iter() {
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    let mut sum = 0;
    // TODO: iterate over vec using for loop (consumes vec)
    // for value in vec { sum += value; }

    assert_eq!(sum, 6);
    // vec is no longer valid here
}

fn _13_iter_chain() {
    let vec = my_vec![1, 2, 3, 4, 5];

    // TODO: use into_iter() with filter and map to get doubled even numbers
    let result: MyVec<i32> = my_vec![]; // Should be [4, 8, 16]

    assert_eq!(result[0], 4);
    assert_eq!(result[1], 8);
    assert_eq!(result[2], 16);
}

fn _14_partial_iteration() {
    let vec = my_vec![
        String::from("a"),
        String::from("b"),
        String::from("c"),
        String::from("d")
    ];

    let mut result: MyVec<String> = my_vec![];
    // TODO: iterate over vec but only take first 2 elements using .take(2)
    // The remaining elements should be automatically dropped

    assert_eq!(result[0], "a");
    assert_eq!(result[1], "b");
    assert_eq!(result.len(), 2);
    // "c" and "d" were dropped by the iterator
}

fn _15_enumerate() {
    let vec = my_vec![10, 20, 30];

    let mut indices: MyVec<usize> = my_vec![];
    // TODO: iterate with enumerate() to get (index, value) pairs
    // Push only the indices where value > 15

    assert_eq!(indices[0], 1);
    assert_eq!(indices[1], 2);
}

fn _16_my_vec_macro() {
    let vec1: MyVec<i32> = MyVec::new(); // TODO: create vec with elements 1, 2, 3 using my_vec! macro

    let vec2: MyVec<i32> = MyVec::new(); // TODO: create vec with 5 copies of 0 using my_vec! macro

    assert_eq!(vec1.len(), 3);
    assert_eq!(vec1[1], 2);
    assert_eq!(vec2.len(), 5);
    assert_eq!(vec2[0], 0);
}

fn _17_growth_strategy() {
    let mut vec = MyVec::new();

    let cap0 = vec.capacity();

    vec.push(1);
    let cap1 = vec.capacity();

    vec.push(2);
    let cap2 = vec.capacity();

    vec.push(3);
    let cap3 = vec.capacity();

    // Vec doubles capacity: 0 -> 1 -> 2 -> 4
    assert_eq!(cap0, 0);
    assert_eq!(cap1, 1);
    assert_eq!(cap2, 2);
    assert_eq!(cap3, 4);
}

// ============================================================================
// Real-world Demo: Building a Dynamic Buffer
// ============================================================================

struct Buffer<T> {
    data: MyVec<T>,
}

impl<T> Buffer<T> {
    fn new() -> Self {
        Buffer { data: MyVec::new() }
    }

    fn push(&mut self, item: T) {
        self.data.push(item);
    }

    fn drain(&mut self) -> MyVec<T> {
        let mut new_data = MyVec::new();
        std::mem::swap(&mut self.data, &mut new_data);
        new_data
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

fn _18_real_world() {
    let mut buffer: Buffer<String> = Buffer::new(); // TODO: create new buffer

    // TODO: push "hello", "world", "!" to buffer

    let len = 0; // TODO: get buffer length
    assert_eq!(len, 3);

    let drained: MyVec<String> = MyVec::new(); // TODO: drain the buffer

    assert_eq!(drained.len(), 3);
    assert_eq!(buffer.len(), 0);
    assert_eq!(drained[0], "hello");
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    run_all![
        "MyVec",
        _01_new_and_push,
        _02_with_capacity,
        _03_pop,
        _04_indexing,
        _05_insert,
        _06_remove,
        _07_clear,
        _08_shrink_to_fit,
        _09_as_slice,
        _10_deref_coercion,
        _11_clone,
        _12_into_iter,
        _13_iter_chain,
        _14_partial_iteration,
        _15_enumerate,
        _16_my_vec_macro,
        _17_growth_strategy,
        _18_real_world,
    ];
}
