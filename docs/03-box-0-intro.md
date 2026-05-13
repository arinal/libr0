# Chapter 3: Box - Heap Allocation

## Common Misconceptions

If you come from Java or C#, you might think this allocates on the heap:

```rust
struct Point { x: i32, y: i32 }

impl Point {
    fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
}

let p = Point::new(3, 2);  // Is this on the heap?
```

**No.** In Rust, `new` is just a method name - there's nothing special about it. The `p: Point` lives on the **stack**.

Another surprise: arrays are also on the stack:

```rust
let arr = [0u8; 1000];  // 1000 bytes on the STACK, not heap!
```

## Heap Allocation Across Languages

In many languages, heap allocation happens through keywords:

- **Java/C#**: `new` is a keyword that allocates on the heap
- **JavaScript**: Creating objects/arrays automatically uses the heap
- **Python**: All objects live on the heap

In **C**, heap allocation is a function call:

- `malloc()`, `calloc()`, `free()` - explicit function calls

In **Rust**, there's **no keyword** for heap allocation. Instead, it's wrapped in types:

- `Box::new()` - allocate a single value
- `Vec::new()` - allocate a growable array
- `String::new()` - allocate a growable string

The raw allocation functions (`alloc`, `dealloc`) are **unsafe** and require manual memory management. You're not supposed to call them directly.

**Key insight**: All methods that internally call `alloc` are doing heap allocation. `Box::new()`, `Vec::push()`, `String::from()` - they all ultimately call `alloc` underneath, but wrap it in safe APIs that handle deallocation automatically.

`Box` is the simplest and most direct safe wrapper around heap allocation.

## Stack vs Heap

When you create a variable in Rust, it lives on the **stack** by default:

```rust
let x = 42;           // 4 bytes on stack
let y = [0u8; 1000];  // 1000 bytes on stack (!)
let p = Point::new(3, 2);  // 8 bytes on stack
```

Stack allocation is fast but limited:

- Size must be known at compile time
- Data is dropped when the function returns
- Stack space is limited (typically 1-8 MB)

The **heap** is for dynamic allocation:

- Size can be determined at runtime
- Data lives until explicitly freed
- Much larger (limited by RAM)
- **Requires explicit action in Rust** (via `Box`, `Vec`, `String`, etc.)

## What is Box?

`Box<T>` is the simplest smart pointer. It:

1. Allocates memory on the heap
2. Stores a value there
3. Keeps a pointer to that memory on the stack
4. Automatically frees the memory when dropped

```bob
   STACK        |       HEAP
                |
+---------+     |     +---------+
| Box<T>  |     |     |    T    |
+---------+     |     +---------+
|  ptr *--+-----+---->|  value  |
+---------+     |     +---------+
  8 bytes       |       size of T
                |

```

## Why Use Box?

### 1. Recursive Types

This won't compile:

```rust
enum List {
    Cons(i32, List),  // Error: couldn't figure out the layout of recursive
    Nil,
}
```

**Memory layout without Box (infinite, this confuses the compiler!):**

```bob
+-------------------------------+
| Cons                          |
+-----+-------------------------+ 
| i32 | Cons                    | 
|     +-------------------------+ 
|     | i32 | Cons              | 
|     |     +-----+-------------+ 
|     |     | i32 | Cons        | 
|     |     |     +-----+-------+ 
|     |     |     | i32 | Cons  | 
|     |     |     |     +-------+
|     |     |     |     | ...   |
+-----+-----+-----+-----+-------+

  ... infinite nesting!
```

The compiler tries to calculate: `size(List) = 4 + size(List) = 4 + 4 + size(List) = ...` - it never ends.

Fixed with `Box`:

```rust
enum List {
    Cons(i32, Box<List>),  // Box has known size (pointer)
    Nil,
}
```

**Why doesn't `Box<List>` have the same problem?**

Look at what `Box` actually is:

```rust
struct Box<T> {
    ptr: *mut T,  // Just a pointer! T is not stored here.
}
```

The `T` in `Box<T>` is only a **generic parameter** - it tells the compiler what type the pointer points to, but `T` is never a field inside `Box`. The struct is always just a pointer (8 bytes).

So `Box<List>` doesn't contain a `List`. It contains a _pointer_ to a `List` somewhere. The size of `Box<List>` is always 8 bytes, regardless of what `List` is.


**Memory layout with Box, fixed!:**

```bob
        STACK              |              HEAP
                           |
+---------------------+    |    +---------------------+
| Cons                |    |    | Cons                |
+---------------------+    |    +---------------------+
| i32 "(4 bytes)"     |    |    | i32: 4 bytes        |
+---------------------+    |    +---------------------+    +---------------------+
| ptr: *--------------+----+--->| ptr: *--------------+--->| List::Nil           |
|  "(8 bytes)"        |    |    |  "(8 bytes)"        |    +---------------------+
+---------------------+    |    +---------------------+   
| Total: 12 bytes     |    |    | Total: 12 bytes     |   
+---------------------+    |    +---------------------+   
                           |
                           |
                           |
```

The arrows show where each pointer **points to** in memory (addresses like `0x1000`, `0x2000`). The Box itself is just 8 bytes storing an address.

Now the compiler knows: `size(List) = max(size(Cons), size(Nil)) = max(4 + 8, 0) = 12 bytes`. Done!

### 2. Large Data

In Rust, **move = memcpy**. When you pass a value to a function or assign it to another variable, Rust copies the bytes:

```rust
let huge = [0u8; 1_000_000];  // 1MB array on stack

fn process(data: [u8; 1_000_000]) { /* ... */ }

process(huge);  // Copies 1MB of bytes to the function's stack frame!
```

With Box, the large data lives on the heap. Only the pointer (8 bytes) is on the stack:

```rust
let boxed = Box::new([0u8; 1_000_000]);  // 1MB on heap, 8-byte ptr on stack

fn process(data: Box<[u8; 1_000_000]>) { /* ... */ }

process(boxed);  // Copies only 8 bytes (the pointer), not 1MB!
```

The heap data stays in place. Only the pointer moves.

### 3. Trait Objects (Dynamic Dispatch)

Sometimes you want to return different types that implement the same trait. Without `Box`, this is impossible:

```rust
trait Animal {
    fn sound(&self) -> &str;
}

struct Dog;
struct Cat;

impl Animal for Dog {
    fn sound(&self) -> &str { "Woof!" }
}

impl Animal for Cat {
    fn sound(&self) -> &str { "Meow!" }
}

// ❌ This doesn't work - different return types!
fn make_animal(dog: bool) -> ??? {
    if dog {
        Dog  // Type: Dog (size: 0 bytes)
    } else {
        Cat  // Type: Cat (size: 0 bytes)
    }
}
```

**The problem:** Functions must have a single, known return type. `Dog` and `Cat` are different types, even if they both implement `Animal`.

**Solution 1: Generic (Static Dispatch)** - Doesn't work here:

```rust
// ❌ Won't compile - can't return different types
fn make_animal<T: Animal>(dog: bool) -> T {
    if dog {
        Dog  // T must be Dog
    } else {
        Cat  // T must be Cat - conflict!
    }
}
```

The problem: `T` is a single concrete type chosen by the caller, but we're trying to return two different types based on runtime logic.

**Solution 2: Trait Objects (Dynamic Dispatch)** - Use `Box<dyn Animal>`:

```rust
// ✅ Works! Returns a trait object
fn make_animal(dog: bool) -> Box<dyn Animal> {
    if dog {
        Box::new(Dog)  // Box<Dog> → Box<dyn Animal>
    } else {
        Box::new(Cat)  // Box<Cat> → Box<dyn Animal>
    }
}

// Usage
let animal = make_animal(true);
animal.sound()  // "Woof!" - decided at runtime
```

**Key takeaway:** Use `Box<dyn Trait>` when you need to:

- Return different types from the same function
- Store different types in the same collection
- Decide which type to use at runtime (plugins, configuration, user input)

Use generics (`<T: Trait>`) when you:

- Know the type at compile time
- Want maximum performance
- Don't need to mix different types

