# Building Rust's Standard Library from Scratch

A hands-on learning guide to understanding Rust's core types by implementing them ourselves.

## Philosophy

The best way to understand how something works is to build it yourself. This guide takes you through implementing Rust's fundamental types from scratch, starting with simple enums and progressing to async runtime internals.

Each chapter builds on the previous one, introducing new concepts gradually.

This guide emphasizes memory layout - pointers, stack, and heap. Understanding process memory layout will help you see how these types actually work in memory. Understanding blocking vs non-blocking operations is also helpful when we implement the event loop using epoll.

For these low-level concepts, refer to: https://github.com/arinal/under-the-hood

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

3.5. **[Vec](./03.5-vec.md)** - Growable arrays
   - Heap allocation without `Box`
   - Direct use of allocator APIs (alloc/dealloc)
   - `ptr`, `len`, and `capacity`
   - Growing and shrinking
   - `String` is just `Vec<u8>`
   - Slices: `&[T]` and `&str`

4. **[Cell](./04-cell.md)** - Copy-based interior mutability
   - What is interior mutability?
   - `UnsafeCell` - the foundation
   - `get` and `set` operations
   - When to use `Cell`

5. **[RefCell](./05-refcell.md)** - Runtime borrow checking
   - `borrow()` and `borrow_mut()`
   - `BorrowError` and `BorrowMutError`
   - The `Ref` and `RefMut` guard types
   - Dynamic borrow checking

6. **[Rc](./06-rc.md)** - Reference counting
   - Shared ownership
   - Uses `Cell` internally for the count!
   - Weak references and cycles
   - Reference counting patterns

7. **[Arc](./07-arc.md)** - Atomic reference counting
   - Thread-safe reference counting
   - Atomic operations
   - When to use `Arc` vs `Rc`

### Part 3: Core Traits & Concepts

8. **[Send and Sync](./08-send-sync.md)** - Thread safety markers
   - What are marker traits?
   - `Send` - safe to transfer between threads
   - `Sync` - safe to share between threads
   - Auto-trait implementation rules
   - Why `Rc` is not `Send` or `Sync`
   - Why `Arc` is `Send + Sync`

9. **[Closures](./09-closures.md)** - Function-like types
   - `Fn` - immutable capture
   - `FnMut` - mutable capture
   - `FnOnce` - consuming capture
   - Closure capture mechanics
   - When to use each trait

10. **[Dynamic Dispatch](./10-dynamic-dispatch.md)** - Runtime polymorphism
    - Static vs Dynamic dispatch
    - Trait objects (`dyn Trait`)
    - Fat pointers (data + vtable)
    - Object safety rules
    - Performance trade-offs

### Part 4: Concurrency Primitives

11. **[Mutex](./11-mutex.md)** - Mutual exclusion
    - Interior mutability for threads
    - Lock guards and RAII
    - Poisoning on panic
    - Deadlock prevention

12. **[Channels](./12-channels.md)** - Message passing
    - `mpsc` - multi-producer, single-consumer
    - `Sender` and `Receiver`
    - Synchronous vs asynchronous channels
    - Channel patterns

### Part 5: Combining Patterns

13. **[Rc + RefCell](./13-rc-refcell.md)** - Shared mutable state (single-threaded)
    - Building a simple graph structure
    - Common patterns and pitfalls

14. **[Arc + Mutex](./14-arc-mutex.md)** - Shared mutable state (multi-threaded)
    - Thread-safe shared state
    - Lock contention
    - When to use `RwLock` instead

### Part 6: Async Rust

15. **[Async State Machines](./15-async-state-machine.md)** - How async/await works
    - Desugaring `async fn`
    - State machine transformation
    - Why we need `Pin`

16. **[Future Trait](./16-future.md)** - The async foundation
    - The `Future` trait
    - `Poll::Ready` and `Poll::Pending`
    - Implementing a simple future
    - Composing futures

17. **[Waker](./17-waker.md)** - Wake-up mechanism
    - How tasks get notified
    - `RawWaker` and `RawWakerVTable`
    - Building a simple waker
    - Wake-on-ready pattern

18. **[Mini Executor](./18-executor.md)** - Running futures
    - Single-threaded executor
    - Task queue and scheduling
    - `block_on` implementation
    - Task spawning

19. **[Mini Tokio](./19-mini-tokio.md)** - Async runtime from scratch
    - Event loop with epoll/kqueue
    - Multi-threaded work-stealing scheduler
    - Timer wheel implementation
    - TCP listener and streams
    - Putting it all together

## Prerequisites

- Basic Rust syntax (functions, structs, enums)
- Understanding of ownership and borrowing
- Familiarity with generics and traits

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
