//! Chapter 3.5: Vec - Growable Arrays
//!
//! Run with: cargo run --example 03_5_vec

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr;

pub struct MyVec<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> MyVec<T> {
    pub fn new() -> MyVec<T> {
        MyVec {
            ptr: std::ptr::NonNull::dangling().as_ptr(),
            len: 0,
            capacity: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> MyVec<T> {
        if capacity == 0 {
            return MyVec::new();
        }

        let layout = Layout::array::<T>(capacity).unwrap();
        let ptr = unsafe { alloc(layout) as *mut T };

        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }

        MyVec {
            ptr,
            len: 0,
            capacity,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        unsafe { Some(ptr::read(self.ptr.add(self.len))) }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        if index > self.len {
            panic!("insert index out of bounds: {} > {}", index, self.len);
        }

        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            // Shift elements to the right
            ptr::copy(
                self.ptr.add(index),
                self.ptr.add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr.add(index), value);
        }
        self.len += 1;
    }

    pub fn remove(&mut self, index: usize) -> T {
        if index >= self.len {
            panic!("remove index out of bounds: {} >= {}", index, self.len);
        }

        unsafe {
            let value = ptr::read(self.ptr.add(index));
            // Shift elements to the left
            ptr::copy(
                self.ptr.add(index + 1),
                self.ptr.add(index),
                self.len - index - 1,
            );
            self.len -= 1;
            value
        }
    }

    pub fn clear(&mut self) {
        if self.len > 0 {
            unsafe {
                ptr::drop_in_place(std::slice::from_raw_parts_mut(self.ptr, self.len));
            }
            self.len = 0;
        }
    }

    pub fn shrink_to_fit(&mut self) {
        if self.capacity == self.len {
            return;
        }

        if self.len == 0 {
            if self.capacity > 0 {
                unsafe {
                    let layout = Layout::array::<T>(self.capacity).unwrap();
                    dealloc(self.ptr as *mut u8, layout);
                }
            }
            self.ptr = std::ptr::NonNull::dangling().as_ptr();
            self.capacity = 0;
            return;
        }

        let new_layout = Layout::array::<T>(self.len).unwrap();
        let old_layout = Layout::array::<T>(self.capacity).unwrap();

        let new_ptr = unsafe {
            realloc(self.ptr as *mut u8, old_layout, new_layout.size()) as *mut T
        };

        if new_ptr.is_null() {
            std::alloc::handle_alloc_error(new_layout);
        }

        self.ptr = new_ptr;
        self.capacity = self.len;
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            1
        } else {
            self.capacity * 2
        };

        let new_layout = Layout::array::<T>(new_capacity).unwrap();

        let new_ptr = if self.capacity == 0 {
            unsafe { alloc(new_layout) as *mut T }
        } else {
            let old_layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                realloc(self.ptr as *mut u8, old_layout, new_layout.size()) as *mut T
            }
        };

        if new_ptr.is_null() {
            std::alloc::handle_alloc_error(new_layout);
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

impl<T> Index<usize> for MyVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        if index >= self.len {
            panic!("index out of bounds: {} >= {}", index, self.len);
        }
        unsafe { &*self.ptr.add(index) }
    }
}

impl<T> IndexMut<usize> for MyVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        if index >= self.len {
            panic!("index out of bounds: {} >= {}", index, self.len);
        }
        unsafe { &mut *self.ptr.add(index) }
    }
}

impl<T> Drop for MyVec<T> {
    fn drop(&mut self) {
        if self.capacity > 0 {
            unsafe {
                ptr::drop_in_place(std::slice::from_raw_parts_mut(self.ptr, self.len));
                let layout = Layout::array::<T>(self.capacity).unwrap();
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl<T> Deref for MyVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for MyVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Clone> Clone for MyVec<T> {
    fn clone(&self) -> MyVec<T> {
        let mut new_vec = MyVec::with_capacity(self.len);
        for i in 0..self.len {
            new_vec.push(self[i].clone());
        }
        new_vec
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for MyVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

// ============================================================================
// IntoIterator implementation
// ============================================================================

pub struct MyVecIntoIter<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
    index: usize,
}

impl<T> Iterator for MyVecIntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let value = unsafe { ptr::read(self.ptr.add(self.index)) };
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.len - self.index;
        (remaining, Some(remaining))
    }
}

impl<T> Drop for MyVecIntoIter<T> {
    fn drop(&mut self) {
        // Drop remaining elements that weren't consumed
        while self.index < self.len {
            unsafe {
                ptr::drop_in_place(self.ptr.add(self.index));
            }
            self.index += 1;
        }
        // Deallocate memory
        if self.capacity > 0 {
            unsafe {
                let layout = Layout::array::<T>(self.capacity).unwrap();
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

impl<T> IntoIterator for MyVec<T> {
    type Item = T;
    type IntoIter = MyVecIntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = MyVecIntoIter {
            ptr: self.ptr,
            len: self.len,
            capacity: self.capacity,
            index: 0,
        };
        // Prevent the original vec from dropping
        std::mem::forget(self);
        iter
    }
}

// ============================================================================
// Demo functions
// ============================================================================

fn _01_basic_usage() {
    println!("--- Basic Usage ---");
    let mut vec = MyVec::new();
    println!("New vec: len={}, capacity={}", vec.len(), vec.capacity());

    vec.push(10);
    vec.push(20);
    vec.push(30);
    println!("After 3 pushes: len={}, capacity={}", vec.len(), vec.capacity());
    println!("vec[0] = {}", vec[0]);
    println!("vec[1] = {}", vec[1]);
    println!("vec[2] = {}", vec[2]);
}

fn _02_growth_strategy() {
    println!("\n--- Growth Strategy ---");
    let mut vec = MyVec::new();
    println!("Start: len={}, capacity={}", vec.len(), vec.capacity());

    for i in 1..=10 {
        vec.push(i);
        println!(
            "After push({}): len={}, capacity={}",
            i,
            vec.len(),
            vec.capacity()
        );
    }
}

fn _03_pop() {
    println!("\n--- Pop ---");
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    println!("Vec: {:?}", vec);

    println!("pop() = {:?}", vec.pop());
    println!("pop() = {:?}", vec.pop());
    println!("Vec after pops: {:?}", vec);
    println!("Capacity unchanged: {}", vec.capacity());
}

fn _04_indexing() {
    println!("\n--- Indexing ---");
    let mut vec = MyVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);

    println!("vec[0] = {}", vec[0]);
    println!("vec[1] = {}", vec[1]);

    vec[1] = 99;
    println!("After vec[1] = 99:");
    println!("vec[1] = {}", vec[1]);
}

fn _05_with_capacity() {
    println!("\n--- with_capacity ---");
    let mut vec = MyVec::with_capacity(10);
    println!("with_capacity(10): len={}, capacity={}", vec.len(), vec.capacity());

    for i in 0..10 {
        vec.push(i);
    }
    println!("After 10 pushes: len={}, capacity={}", vec.len(), vec.capacity());
    println!("No reallocation needed!");
}

fn _06_insert() {
    println!("\n--- insert ---");
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(3);
    vec.push(4);
    println!("Before insert: {:?}", vec);

    vec.insert(1, 2);
    println!("After insert(1, 2): {:?}", vec);

    vec.insert(0, 0);
    println!("After insert(0, 0): {:?}", vec);
}

fn _07_remove() {
    println!("\n--- remove ---");
    let mut vec = MyVec::new();
    vec.push(10);
    vec.push(20);
    vec.push(30);
    vec.push(40);
    println!("Before remove: {:?}", vec);

    let removed = vec.remove(1);
    println!("Removed value: {}", removed);
    println!("After remove(1): {:?}", vec);
}

fn _08_clear() {
    println!("\n--- clear ---");
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    println!("Before clear: len={}, capacity={}", vec.len(), vec.capacity());

    vec.clear();
    println!("After clear: len={}, capacity={}", vec.len(), vec.capacity());
    println!("Capacity remains allocated");
}

fn _09_shrink_to_fit() {
    println!("\n--- shrink_to_fit ---");
    let mut vec = MyVec::new();
    for i in 0..10 {
        vec.push(i);
    }
    println!("After pushes: len={}, capacity={}", vec.len(), vec.capacity());

    vec.pop();
    vec.pop();
    vec.pop();
    println!("After 3 pops: len={}, capacity={}", vec.len(), vec.capacity());

    vec.shrink_to_fit();
    println!("After shrink_to_fit: len={}, capacity={}", vec.len(), vec.capacity());
}

fn _10_as_slice() {
    println!("\n--- as_slice ---");
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);

    let slice = vec.as_slice();
    println!("Slice length: {}", slice.len());
    println!("Slice[0]: {}", slice[0]);
    println!("Slice iter: {:?}", slice.iter().collect::<Vec<_>>());
}

fn _11_deref_coercion() {
    println!("\n--- Deref Coercion ---");
    let mut vec = MyVec::new();
    vec.push(3);
    vec.push(1);
    vec.push(2);
    println!("Before sort: {:?}", vec);

    // This calls [T]::sort() via Deref!
    vec.sort();
    println!("After sort: {:?}", vec);

    // Other slice methods work too
    println!("First: {:?}", vec.first());
    println!("Last: {:?}", vec.last());
    println!("Contains 2: {}", vec.contains(&2));
}

fn _12_clone() {
    println!("\n--- Clone ---");
    let mut vec1 = MyVec::new();
    vec1.push(String::from("hello"));
    vec1.push(String::from("world"));
    println!("vec1: {:?}", vec1);

    let vec2 = vec1.clone();
    println!("vec2 (cloned): {:?}", vec2);
    println!("Both vecs are independent");
}

fn _13_drop_demo() {
    println!("\n--- Drop Demo ---");
    println!("Creating vec with String elements...");
    {
        let mut vec = MyVec::new();
        vec.push(String::from("hello"));
        vec.push(String::from("world"));
        println!("Vec: {:?}", vec);
        println!("Vec going out of scope...");
    }
    println!("Vec dropped - both Strings and memory deallocated");
}

fn _14_string_analogy() {
    println!("\n--- String is Vec<u8> ---");

    // String is essentially:
    // struct String { vec: Vec<u8> }

    let s = String::from("hello");
    println!("String: \"{}\"", s);
    println!("As bytes: {:?}", s.as_bytes());
    println!("Length: {}", s.len());

    // String and Vec<u8> have the same memory layout
    println!("\nString has same (ptr, len, capacity) structure as Vec");
    println!("&str is like &[u8] but with UTF-8 guarantee");
}

fn _15_practical_usage() {
    println!("\n--- Practical: Building a Dynamic List ---");
    let mut numbers = MyVec::new();

    // Collect even numbers from 1 to 20
    for i in 1..=20 {
        if i % 2 == 0 {
            numbers.push(i);
        }
    }

    println!("Even numbers: {:?}", numbers);
    println!("Sum: {}", numbers.iter().sum::<i32>());
    println!("Max: {:?}", numbers.iter().max());

    // Remove numbers divisible by 4
    let mut i = 0;
    while i < numbers.len() {
        if numbers[i] % 4 == 0 {
            numbers.remove(i);
        } else {
            i += 1;
        }
    }
    println!("After removing multiples of 4: {:?}", numbers);
}

fn _16_memory_layout() {
    println!("\n--- Memory Layout ---");
    let mut vec = MyVec::new();
    println!("Empty vec:");
    println!("  Stack size: {} bytes (ptr + len + capacity)",
             std::mem::size_of::<MyVec<i32>>());
    println!("  Heap allocation: 0 bytes");
    println!("  len={}, capacity={}", vec.len(), vec.capacity());

    vec.push(1);
    vec.push(2);
    vec.push(3);
    println!("\nVec with 3 elements:");
    println!("  Stack size: {} bytes (unchanged)",
             std::mem::size_of::<MyVec<i32>>());
    println!("  Heap allocation: {} bytes ({} elements × {} bytes)",
             vec.capacity() * std::mem::size_of::<i32>(),
             vec.capacity(),
             std::mem::size_of::<i32>());
    println!("  len={}, capacity={}", vec.len(), vec.capacity());
}

fn _17_into_iterator() {
    println!("\n--- IntoIterator ---");
    let mut vec = MyVec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);
    println!("Vec before iteration: {:?}", vec);

    // Consume the vec and iterate over owned values
    println!("Iterating with into_iter():");
    for (i, value) in vec.into_iter().enumerate() {
        println!("  [{}] = {}", i, value);
    }
    println!("Vec is now consumed (moved)");

    // Demonstrate partial iteration with Drop
    let mut vec2 = MyVec::new();
    vec2.push(String::from("a"));
    vec2.push(String::from("b"));
    vec2.push(String::from("c"));

    println!("\nPartial iteration (take 2 of 3):");
    for value in vec2.into_iter().take(2) {
        println!("  Got: {}", value);
    }
    println!("Iterator dropped, remaining element 'c' was cleaned up");
}

fn main() {
    println!("=== MyVec Demo ===\n");

    _01_basic_usage();
    _02_growth_strategy();
    _03_pop();
    _04_indexing();
    _05_with_capacity();
    _06_insert();
    _07_remove();
    _08_clear();
    _09_shrink_to_fit();
    _10_as_slice();
    _11_deref_coercion();
    _12_clone();
    _13_drop_demo();
    _14_string_analogy();
    _15_practical_usage();
    _16_memory_layout();
    _17_into_iterator();

    println!("\n=== End Demo ===");
}