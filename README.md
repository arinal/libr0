# libr0

Building Rust's standard library from scratch - a hands-on learning guide to understanding Rust's core types by implementing them ourselves.

## What's in the name?

**libr0** = **lib** **r**ust from zer**0**

We're building Rust's library (lib) from zero, one type at a time.

## Approach

**Learn by building.** The best way to understand something is to implement it yourself. This guide takes you through Rust's fundamental types from scratch, starting with simple enums and progressing to async runtime internals. Each chapter builds on the previous one.

**De-abstract the abstractions.** Rust concepts aren't magic:
- Borrowing? Just a safe pointer (one pointer in memory)
- Moving? Just a copy (mov instruction or memcpy)
- `Box`? Just alloc + dealloc wrapped in a struct
- `RefCell`? Just a counter and panic! when rules break
- `async`? Compiler-generated state machines

**Show the underlying reality:**
- What's in memory? Show the layout
- What does the CPU do? Explain the instruction or syscall
- What does the compiler generate? Show the desugaring

**Concrete over abstract.** When abstractions are hard to grasp, we use good representative examples. Understanding comes from working code, not theory.

**Address common confusions.** If people commonly misunderstand something, we call it out explicitly.

**Recommended resources:**

Since this guide relies heavily on memory layout, this is a great reference for visualizing each type in Rust:
- [cheats.rs/#memory-layout](https://cheats.rs/#memory-layout) - Visual memory layouts for Rust types

For low-level concepts (syscalls, process memory, blocking vs non-blocking operations):
- [under-the-hood](https://github.com/arinal/under-the-hood)

## What You'll Discover

- **Rust has no null** - The language itself doesn't have null. `Option<T>` is just a regular enum we can implement ourselves.
- **String is just a struct** - `String`, `Vec`, `Box`, `Rc` - they're all normal structs you can build yourself. Nothing magical.
- **Rust doesn't know what "threads" or "tasks" are** - The language only knows `Send` and `Sync` traits. Thread safety comes from library functions (often called `spawn`) requiring these traits in their signatures. Threads and tasks? Just library constructs.
- **Heap vs Stack** - Creating a struct or array? That's on the stack. Unsafe allocator calls (`alloc`/`dealloc`) put things on the heap. `Box`, `Vec`, and `Rc` are just safe wrappers.
- **Cell doesn't allocate** - `Cell<T>` and `RefCell<T>` live wherever you put them - stack, heap, or inside other structs. They're just wrappers.
- **PhantomData isn't scary** - Go to definition on stdlib types and you'll see it everywhere. What is it? Just a lifetimes marker. That's it. Nothing magical.
- **Slices are special** - Unlike everything else here, `&[T]` is a language primitive. You can't fully replicate it in user code.
- **String is literally Vec\<u8\>** - It's just a `Vec` with a UTF-8 guarantee. Same memory layout, same `ptr/len/capacity`.
- **async is rewriting your code** - `async fn` desugars into a state machine. The compiler transforms your code completely.
- **No async runtime included** - Rust only provides the `Future` trait. Tokio? That's just a library anyone can write.

## Table of Contents

### Part 1: Foundational Types

1. **[Option](./docs/01-option.md)** - The simplest enum
   - Pattern matching basics
   - `Some(T)` and `None` variants
   - Implementing `map`, `and_then`, `unwrap_or`, `filter`, `as_ref`, `take`

2. **[Result](./docs/02-result.md)** - Error handling
   - `Ok(T)` and `Err(E)` variants
   - The `?` operator
   - Converting between `Option` and `Result`
   - Error propagation patterns

### Part 2: Smart Pointers & Interior Mutability

3. **[Box](./docs/03-box.md)** - Heap allocation
   - Stack vs Heap
   - The `Deref` and `DerefMut` traits
   - `Drop` for cleanup
   - Why recursive types need `Box`
   - Trait objects basics

4. **[Vec](./docs/04-vec.md)** - Growable arrays
   - Heap allocation without `Box`
   - Direct use of allocator APIs (alloc/dealloc)
   - `ptr`, `len`, and `capacity`
   - Growing and shrinking
   - `String` is just `Vec<u8>`
   - Slices: `&[T]` and `&str`

5. **[Cell](./docs/05-cell.md)** - Copy-based interior mutability
   - What is interior mutability?
   - `UnsafeCell` - the foundation
   - `get` and `set` operations
   - When to use `Cell`

6. **[RefCell](./docs/06-refcell.md)** - Runtime borrow checking
   - Interior mutability for any type
   - Guard types (`Ref` and `RefMut`)
   - Runtime borrow tracking with counters
   - RAII and automatic cleanup
   - `try_borrow` for non-panicking access
   - Common pitfalls and patterns

7. **[Rc](./docs/07-rc.md)** - Reference counting for shared ownership
   - Multiple owners of the same data
   - Strong and weak references
   - Breaking reference cycles with `Weak`
   - `Clone` semantics for reference counting
   - When to use `Rc` vs borrowing

8. **[Rc + RefCell](./docs/08-rc-refcell.md)** - Shared mutable state (single-threaded)
   - Combining reference counting with interior mutability
   - Building graph structures with cycles
   - Tree with parent pointers using `Weak`
   - Observer pattern implementation
   - Common patterns and pitfalls
   - Memory leak prevention

### Appendix

- **[Closures](./docs/appendix-closures.md)** - Function-like types
  - `Fn` - immutable capture
  - `FnMut` - mutable capture
  - `FnOnce` - consuming capture
  - Closure capture mechanics
  - When to use each trait

- **[Memory Layout](./docs/appendix-memory-layout.md)** - Where your data lives
  - Process memory layout (Stack, Heap, Static data)
  - Function calls and stack frames
  - Raw pointers and heap allocation
  - Memory layout visualizations

- **[Sized](./docs/appendix-sized.md)** - The `Sized` trait
  - What is `Sized`?
  - Dynamically sized types
  - `?Sized` bounds

### Coming Soon

The following chapters are planned but not yet implemented. See [CLAUDE.md](./CLAUDE.md) for the full roadmap.

- **Arc** - Atomic reference counting
- **Send and Sync** - Thread safety markers
- **Dynamic Dispatch** - Runtime polymorphism
- **Mutex** - Mutual exclusion
- **Channels** - Message passing
- **Arc + Mutex** - Shared mutable state (multi-threaded)
- **Async State Machines** - How async/await works
- **Future Trait** - The async foundation
- **Waker** - Wake-up mechanism
- **Mini Executor** - Running futures
- **Mini Tokio** - Async runtime from scratch

## Prerequisites

- Basic Rust syntax (functions, structs, enums)
- Understanding of ownership and borrowing
- Familiarity with generics and traits

## Helpful Resources

- **[Rust Language Cheat Sheet](https://cheats.rs/)** - Quick reference for Rust syntax and concepts
- **[under-the-hood](https://github.com/arinal/under-the-hood)** - Low-level concepts: memory layout, syscalls, blocking vs non-blocking

## How to Use This Guide

Each chapter contains:

- **Concept explanation** - Why this type exists
- **Step-by-step implementation** - Building it from scratch
- **Code examples** - Practical usage
- **Exercises** - Reinforce your understanding

```rust
// Example: Our own Option type
enum Option0<T> {
    Some(T),
    None,
}

impl<T> Option0<T> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Option0<U> {
        match self {
            Option0::Some(x) => Option0::Some(f(x)),
            Option0::None => Option0::None,
        }
    }
}
```

## Project Structure

```
rustlib/
├── docs/                     # Chapter documentation
│   ├── 01-option.md          # Chapter 1: Option
│   ├── 02-result.md          # Chapter 2: Result
│   ├── 03-box.md             # Chapter 3: Box
│   ├── 04-vec.md             # Chapter 4: Vec
│   ├── 05-cell.md            # Chapter 5: Cell
│   ├── 06-refcell.md         # Chapter 6: RefCell
│   ├── 07-rc.md              # Chapter 7: Rc
│   ├── 08-rc-refcell.md      # Chapter 8: Rc + RefCell
│   └── appendix-closures.md  # Appendix: Closures
├── src/                      # Library implementations
│   ├── lib.rs                # Main library file
│   ├── option.rs             # Option0<T> implementation
│   ├── result.rs             # Result0<T, E> implementation
│   ├── box.rs                # Box0<T> implementation
│   ├── vec.rs                # Vec0<T> implementation
│   ├── cell.rs               # Cell0<T> implementation
│   ├── refcell.rs            # RefCell0<T> implementation
│   └── rc.rs                 # Rc0<T> and Weak0<T> implementation
└── README.md                 # This file
```

## Running the Code

### Running Examples

Each chapter has corresponding example code in the `examples/` directory:

```bash
# Run individual examples (Chapters 1-8 completed)
cargo run --example 01_option      # Chapter 1: Option
cargo run --example 02_result      # Chapter 2: Result
cargo run --example 03_box         # Chapter 3: Box
cargo run --example 04_vec         # Chapter 4: Vec
cargo run --example 05_cell        # Chapter 5: Cell
cargo run --example 06_refcell     # Chapter 6: RefCell
cargo run --example 07_rc          # Chapter 7: Rc
cargo run --example 08_rc_refcell  # Chapter 8: Rc + RefCell
```

**Note:** Example files contain TODO placeholders for you to fill in as exercises. Complete solutions are available on the `answers` branch:

```bash
# Switch to the answers branch to see complete solutions
git checkout answers

# Return to the main branch with exercises
git checkout main
```

### Running Tests

The project includes comprehensive tests:

```bash
# Run all tests
cargo test

# Run only unit tests
cargo test --lib

# Run only documentation tests
cargo test --doc

# Run tests for a specific module
cargo test option
cargo test cell
```

### Using as a Library

You can use the completed implementations in your own code:

```rust
// Completed types (Chapters 1-8)
use rustlib::option::{Option0, Some, None};
use rustlib::result::{Result0, Ok, Err};
use rustlib::r#box::Box0;
use rustlib::vec::Vec0;
use rustlib::cell::Cell0;
use rustlib::refcell::RefCell0;
use rustlib::rc::{Rc0, Weak0};

// Use the vec0! macro
use rustlib::vec0;
let v = vec0![1, 2, 3];
```

## License

MIT
