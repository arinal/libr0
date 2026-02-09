//! Chapter 3: Box - Heap Allocation
//!
//! Run with: cargo run --example box

use std::alloc::{alloc, dealloc, Layout};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr;

struct MyBox<T> {
    ptr: *mut T,
}

impl<T> MyBox<T> {
    fn new(value: T) -> MyBox<T> {
        unsafe {
            // Calculate memory layout for T
            let layout = Layout::new::<T>();

            // Allocate memory
            let ptr = alloc(layout) as *mut T;

            if ptr.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            // Write value to allocated memory
            ptr::write(ptr, value);

            MyBox { ptr }
        }
    }

    fn into_inner(self) -> T {
        unsafe {
            // Read the value out
            let value = ptr::read(self.ptr);

            // Deallocate
            let layout = Layout::new::<T>();
            dealloc(self.ptr as *mut u8, layout);

            // Don't run Drop (we already deallocated)
            std::mem::forget(self);

            value
        }
    }

    fn leak(self) -> &'static mut T {
        let ptr = self.ptr;
        std::mem::forget(self); // Don't run Drop
        unsafe { &mut *ptr }
    }

    fn into_raw(self) -> *mut T {
        let ptr = self.ptr;
        std::mem::forget(self); // Don't run Drop
        ptr
    }

    unsafe fn from_raw(ptr: *mut T) -> MyBox<T> {
        MyBox { ptr }
    }

    // Exercise: map
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MyBox<U> {
        let value = self.into_inner();
        MyBox::new(f(value))
    }
}

impl<T> Deref for MyBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for MyBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for MyBox<T> {
    fn drop(&mut self) {
        unsafe {
            // Call destructor on the value
            ptr::drop_in_place(self.ptr);

            // Deallocate the memory
            let layout = Layout::new::<T>();
            dealloc(self.ptr as *mut u8, layout);
        }
    }
}

// Exercise: Debug
impl<T: fmt::Debug> fmt::Debug for MyBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MyBox").field(&**self).finish()
    }
}

// Exercise: Clone
impl<T: Clone> Clone for MyBox<T> {
    fn clone(&self) -> Self {
        MyBox::new((**self).clone())
    }
}

// ============================================================================
// Recursive type example
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
}

impl<T: fmt::Display> List<T> {
    fn display(&self) {
        match self {
            List::Cons(val, rest) => {
                print!("{} -> ", val);
                rest.display();
            }
            List::Nil => println!("Nil"),
        }
    }
}

// ============================================================================
// Demo
// ============================================================================

fn print_len(s: &str) {
    println!("String length: {}", s.len());
}

fn _01_basic_usage() {
    println!("--- Basic Usage ---");
    let b = MyBox::new(42);
    println!("Created MyBox with value: {}", *b);
}

fn _02_deref() {
    println!("\n--- Deref ---");
    let b = MyBox::new(10);
    let value: i32 = *b; // Deref to get value
    println!("Dereferenced value: {}", value);
}

fn _03_deref_mut() {
    println!("\n--- DerefMut ---");
    let mut b = MyBox::new(5);
    println!("Before mutation: {}", *b);
    *b = 100;
    println!("After mutation: {}", *b);
}

fn _04_deref_coercion() {
    println!("\n--- Deref Coercion ---");
    let boxed_string = MyBox::new(String::from("hello"));
    // &MyBox<String> -> &String -> &str automatically!
    print_len(&boxed_string);
}

fn _05_into_inner() {
    println!("\n--- into_inner ---");
    let b = MyBox::new(String::from("owned"));
    let s: String = b.into_inner();
    println!("Extracted: {}", s);
    // b is now invalid (consumed)
}

fn _06_debug() {
    println!("\n--- Debug ---");
    let b = MyBox::new(vec![1, 2, 3]);
    println!("{:?}", b);
}

fn _07_clone() {
    println!("\n--- Clone ---");
    let b1 = MyBox::new(String::from("original"));
    let b2 = b1.clone();
    println!("b1: {:?}", b1);
    println!("b2: {:?}", b2);
}

fn _08_map() {
    println!("\n--- map ---");
    let b = MyBox::new(5);
    let doubled = b.map(|x| x * 2);
    println!("5 mapped to doubled: {:?}", doubled);

    let b = MyBox::new(String::from("hello"));
    let len_box = b.map(|s| s.len());
    println!("\"hello\" mapped to length: {:?}", len_box);
}

fn _09_recursive_list() {
    println!("\n--- Recursive Type (List) ---");
    let list = List::new().prepend(3).prepend(2).prepend(1);
    print!("List: ");
    list.display();
}

fn _10_drop() {
    println!("\n--- Drop ---");
    {
        let _b = MyBox::new(String::from("will be dropped"));
        println!("MyBox exists in this scope");
    } // _b dropped here
    println!("MyBox has been dropped (memory freed)");
}

fn _11_leak() {
    println!("\n--- leak ---");
    let b = MyBox::new(42);
    let leaked: &'static mut i32 = b.leak();
    println!("Leaked value: {}", *leaked);
    *leaked = 999;
    println!("Modified leaked value: {}", *leaked);
    println!("This memory is never freed!");
}

fn _12_raw_pointers() {
    println!("\n--- into_raw / from_raw ---");
    let b = MyBox::new(String::from("raw pointer"));
    let ptr = MyBox::into_raw(b);
    println!("Converted to raw pointer: {:p}", ptr);

    let restored = unsafe { MyBox::from_raw(ptr) };
    println!("Restored from raw pointer: {:?}", restored);
    // restored will be dropped here
}

fn _13_size_comparison() {
    println!("\n--- Size Comparison ---");
    println!("Size of MyBox<[u8; 1000]>: {} bytes", std::mem::size_of::<MyBox<[u8; 1000]>>());
    println!("Size of [u8; 1000]: {} bytes", std::mem::size_of::<[u8; 1000]>());
    println!("MyBox is just a pointer (8 bytes on 64-bit)!");
}

fn main() {
    println!("=== MyBox Demo ===\n");

    _01_basic_usage();
    _02_deref();
    _03_deref_mut();
    _04_deref_coercion();
    _05_into_inner();
    _06_debug();
    _07_clone();
    _08_map();
    _09_recursive_list();
    _10_drop();
    _11_leak();
    _12_raw_pointers();
    _13_size_comparison();

    println!("\n=== End Demo ===");
}
