# libr0

Building Rust's standard library from scratch - a hands-on learning guide to understanding Rust's core types by implementing them ourselves.

## What's in the name?

**libr0** = **lib** **r**ust from zer**0**

We're building Rust's library (lib) from zero, one type at a time.

## Philosophy

The best way to understand how something works is to build it yourself. This guide takes you through implementing Rust's fundamental types from scratch, starting with simple enums and progressing to async runtime internals.

Each chapter builds on the previous one, introducing new concepts gradually.

This guide emphasizes **memory layout** - pointers, stack, and heap. Understanding process memory layout will help you see how these types actually work in memory. Understanding blocking vs non-blocking operations is also helpful when we implement the event loop using epoll.

**Recommended resources:**

- For Rust memory layout visualization: [cheats.rs/#memory-layout](https://cheats.rs/#memory-layout)
- For low-level concepts (syscalls, process memory, epoll): [under-the-hood](https://github.com/arinal/under-the-hood)

**Things that will surprise you:**

- **Rust has no null** - The language itself doesn't have null. `Option<T>` is just a regular enum we can implement ourselves.
- **String is just a struct** - `String`, `Vec`, `Box`, `Rc` - they're all normal structs you can build yourself. Nothing magical.
- **Rust doesn't know "threads"** - The language only knows about `Send` and `Sync` traits. Threads come from the OS.
- **Heap vs Stack** - Creating a struct or array? That's on the stack. `Box`, `Vec`, and `Rc` are what put things on the heap.
- **Cell doesn't allocate** - `Cell<T>` and `RefCell<T>` live wherever you put them - stack, heap, or inside other structs. They're just wrappers.
- **Slices are special** - Unlike everything else here, `&[T]` is a language primitive. You can't fully replicate it in user code.
- **String is literally Vec\<u8\>** - It's just a `Vec` with a UTF-8 guarantee. Same memory layout, same `ptr/len/capacity`.
- **async is rewriting your code** - `async fn` desugars into a state machine. The compiler transforms your code completely.
- **No async runtime included** - Rust only provides the `Future` trait. Tokio? That's just a library anyone can write.

## Table of Contents

### Part 1: Foundational Types

1. **[Option](./01-option.md)** - The simplest enum
   - Pattern matching basics
   - `Some(T)` and `None` variants
   - Implementing `map`, `and_then`, `unwrap_or`, `filter`, `as_ref`, `take`

2. **[Result](./02-result.md)** - Error handling
   - `Ok(T)` and `Err(E)` variants
   - The `?` operator
   - Converting between `Option` and `Result`
   - Error propagation patterns

### Part 2: Smart Pointers & Interior Mutability

3. **[Box](./03-box.md)** - Heap allocation
   - Stack vs Heap
   - The `Deref` and `DerefMut` traits
   - `Drop` for cleanup
   - Why recursive types need `Box`
   - Trait objects basics

4. **[Vec](./04-vec.md)** - Growable arrays
   - Heap allocation without `Box`
   - Direct use of allocator APIs (alloc/dealloc)
   - `ptr`, `len`, and `capacity`
   - Growing and shrinking
   - `String` is just `Vec<u8>`
   - Slices: `&[T]` and `&str`

5. **[Cell](./05-cell.md)** - Copy-based interior mutability
   - What is interior mutability?
   - `UnsafeCell` - the foundation
   - `get` and `set` operations
   - When to use `Cell`

6. **[RefCell](./06-refcell.md)** - Runtime borrow checking
   - `borrow()` and `borrow_mut()`
   - `BorrowError` and `BorrowMutError`
   - The `Ref` and `RefMut` guard types
   - Dynamic borrow checking

7. **[Rc](./07-rc.md)** - Reference counting
   - Shared ownership
   - Uses `Cell` internally for the count!
   - Weak references and cycles
   - Reference counting patterns

8. **[Arc](./08-arc.md)** - Atomic reference counting
   - Thread-safe reference counting
   - Atomic operations
   - When to use `Arc` vs `Rc`

### Part 3: Core Traits & Concepts

9. **[Send and Sync](./09-send-sync.md)** - Thread safety markers
   - What are marker traits?
   - `Send` - safe to transfer between threads
   - `Sync` - safe to share between threads
   - Auto-trait implementation rules
   - Why `Rc` is not `Send` or `Sync`
   - Why `Arc` is `Send + Sync`

10. **[Closures](./10-closures.md)** - Function-like types
   - `Fn` - immutable capture
   - `FnMut` - mutable capture
   - `FnOnce` - consuming capture
   - Closure capture mechanics
   - When to use each trait

11. **[Dynamic Dispatch](./11-dynamic-dispatch.md)** - Runtime polymorphism
    - Static vs Dynamic dispatch
    - Trait objects (`dyn Trait`)
    - Fat pointers (data + vtable)
    - Object safety rules
    - Performance trade-offs

### Part 4: Concurrency Primitives

12. **[Mutex](./12-mutex.md)** - Mutual exclusion
    - Interior mutability for threads
    - Lock guards and RAII
    - Poisoning on panic
    - Deadlock prevention

13. **[Channels](./13-channels.md)** - Message passing
    - `mpsc` - multi-producer, single-consumer
    - `Sender` and `Receiver`
    - Synchronous vs asynchronous channels
    - Channel patterns

### Part 5: Combining Patterns

14. **[Rc + RefCell](./14-rc-refcell.md)** - Shared mutable state (single-threaded)
    - Building a simple graph structure
    - Common patterns and pitfalls

15. **[Arc + Mutex](./15-arc-mutex.md)** - Shared mutable state (multi-threaded)
    - Thread-safe shared state
    - Lock contention
    - When to use `RwLock` instead

### Part 6: Async Rust

16. **[Async State Machines](./16-async-state-machine.md)** - How async/await works
    - Desugaring `async fn`
    - State machine transformation
    - Why we need `Pin`

17. **[Future Trait](./17-future.md)** - The async foundation
    - The `Future` trait
    - `Poll::Ready` and `Poll::Pending`
    - Implementing a simple future
    - Composing futures

18. **[Waker](./18-waker.md)** - Wake-up mechanism
    - How tasks get notified
    - `RawWaker` and `RawWakerVTable`
    - Building a simple waker
    - Wake-on-ready pattern

19. **[Mini Executor](./19-executor.md)** - Running futures
    - Single-threaded executor
    - Task queue and scheduling
    - `block_on` implementation
    - Task spawning

20. **[Mini Tokio](./20-mini-tokio.md)** - Async runtime from scratch
    - Event loop with epoll/kqueue
    - Multi-threaded work-stealing scheduler
    - Timer wheel implementation
    - TCP listener and streams
    - Putting it all together

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
enum MyOption<T> {
    Some(T),
    None,
}

impl<T> MyOption<T> {
    fn map<U, F: FnOnce(T) -> U>(self, f: F) -> MyOption<U> {
        match self {
            MyOption::Some(x) => MyOption::Some(f(x)),
            MyOption::None => MyOption::None,
        }
    }
}
```

## Running the Code

Each chapter has corresponding code in the `examples/` directory:

```bash
cargo run --example option
cargo run --example cell
cargo run --example rc
cargo run --example mini_tokio
```

## License

MIT
