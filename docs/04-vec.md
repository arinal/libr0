# Chapter 4: Vec - Growable Arrays

## The Problem: Fixed-Size Arrays

Arrays in Rust have a fixed size known at compile time:

```rust
let arr: [i32; 3] = [1, 2, 3];
// Can't grow or shrink
```

What if we need a collection that can grow dynamically at runtime?

## Vec: A Growable Array

`Vec<T>` is Rust's dynamically-sized array type. Unlike `Box<T>` which allocates a single value on the heap, `Vec<T>` allocates a _contiguous block_ of memory that can grow or shrink.

## The Three Fields of Vec

```rust
pub struct Vec<T> {
    ptr: *mut T,      // Pointer to heap-allocated array
    len: usize,       // Number of elements currently in use
    capacity: usize,  // Total allocated space (in elements)
}
```

**Key insight:** `len <= capacity` always.

```
Heap memory:
[1, 2, 3, ?, ?, ?]
 ^           ^
 |           |
 len = 3     capacity = 6
```

## Why Not Use Box?

`Box<T>` allocates space for _exactly one_ `T`. To grow, we'd need to:

1. Allocate a new `Box`
2. Copy all elements
3. Deallocate the old `Box`

Instead, `Vec` uses the allocator APIs directly (`alloc`, `realloc`, `dealloc`) to:

- Allocate more space than immediately needed (capacity > len)
- Grow in-place when possible
- Only reallocate when we run out of capacity

## Implementing Vec

### Basic Structure

```rust
use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr;

pub struct Vec0<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> Vec0<T> {
    pub fn new() -> Vec0<T> {
        Vec0 {
            ptr: std::ptr::NonNull::dangling().as_ptr(),
            len: 0,
            capacity: 0,
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
}
```

### Push - Adding Elements

When `len == capacity`, we need to grow:

```rust
impl<T> Vec0<T> {
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            self.grow();
        }

        unsafe {
            // Write to the next available slot
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    fn grow(&mut self) {
        let new_capacity = if self.capacity == 0 {
            1
        } else {
            self.capacity * 2  // Double the capacity
        };

        let new_layout = Layout::array::<T>(new_capacity).unwrap();

        let new_ptr = if self.capacity == 0 {
            // First allocation
            unsafe { alloc(new_layout) as *mut T }
        } else {
            // Reallocate
            let old_layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                realloc(
                    self.ptr as *mut u8,
                    old_layout,
                    new_layout.size(),
                ) as *mut T
            }
        };

        if new_ptr.is_null() {
            std::alloc::handle_alloc_error(new_layout);
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}
```

**Growth strategy:** Start at 1, then double each time.

```
Capacity progression: 0 → 1 → 2 → 4 → 8 → 16 → 32 → ...
```

Why double? Amortized O(1) push operations.

**Note:** The `vec!` macro is syntactic sugar for repeatedly calling `push`:

```rust
let v = vec![1, 2, 3];
// Expands to roughly:
// let mut v = Vec::new();
// v.push(1);
// v.push(2);
// v.push(3);
```

Here's a simplified implementation of the macro:

```rust
#[macro_export]
macro_rules! vec {
    () => {
        Vec::new()
    };
    ($elem:expr; $n:expr) => {
        // vec![0; 5] creates [0, 0, 0, 0, 0]
        {
            let mut v = Vec::with_capacity($n);
            v.resize($n, $elem);
            v
        }
    };
    ($($x:expr),+ $(,)?) => {
        // vec![1, 2, 3]
        {
            let mut v = Vec::new();
            $(v.push($x);)*
            v
        }
    };
}
```

The macro has three patterns:
1. `vec![]` - creates an empty vector
2. `vec![elem; n]` - creates a vector with `n` copies of `elem`
3. `vec![x, y, z]` - creates a vector with the given elements

### Pop - Removing Elements

```rust
impl<T> Vec0<T> {
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        unsafe {
            Some(ptr::read(self.ptr.add(self.len)))
        }
    }
}
```

**Note:** We don't shrink capacity on pop. The memory stays allocated.

### Index Access

```rust
use std::ops::{Index, IndexMut};

impl<T> Index<usize> for Vec0<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        if index >= self.len {
            panic!("index out of bounds: {} >= {}", index, self.len);
        }
        unsafe { &*self.ptr.add(index) }
    }
}

impl<T> IndexMut<usize> for Vec0<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        if index >= self.len {
            panic!("index out of bounds: {} >= {}", index, self.len);
        }
        unsafe { &mut *self.ptr.add(index) }
    }
}
```

Now we can use `vec[i]`:

```rust
let mut vec = Vec0::new();
vec.push(10);
vec.push(20);
vec[0]  // 10
vec[1] = 99;
vec[1]  // 99
```

### Drop Implementation

Critical! We must:

1. Drop all elements (call their destructors)
2. Deallocate the memory

```rust
impl<T> Drop for Vec0<T> {
    fn drop(&mut self) {
        if self.capacity > 0 {
            // Drop all elements
            unsafe {
                ptr::drop_in_place(
                    std::slice::from_raw_parts_mut(self.ptr, self.len)
                );
            }

            // Deallocate memory
            let layout = Layout::array::<T>(self.capacity).unwrap();
            unsafe {
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}
```

## Slices: Views into Vec

**Important:** Unlike `Vec`, `Option`, `Result`, or `Box`, slices (`[T]` and `&[T]`) are a **language primitive** built into the Rust compiler. You cannot implement your own slice type with identical behavior.

Why slices are special:

- `[T]` is a **dynamically sized type (DST)** - no known size at compile time
- The compiler has special knowledge of slices for:
  - Array to slice coercion: `&[1, 2, 3]` automatically becomes `&[i32]`
  - Slice syntax: `&vec[1..3]` uses built-in range operators
  - Pattern matching: `match slice { [first, rest @ ..] => ... }`
  - Indexing bounds checks are optimized by the compiler

**Can we implement something slice-like?** Yes! We can create a struct with `(ptr, len)` that _behaves_ like a slice, but it won't have the same compiler integration. We'll show this in the exercises.

A slice `&[T]` is a _view_ into contiguous memory. It's a fat pointer:

```
Slice structure:
┌───────────────┬───────────┐
│ ptr: *const T │ len: usize│
└──────│────────┴───────────┘
       │
       └──────> [T, T, T] (points to array elements in memory)
```

Convert `Vec<T>` to `&[T]`:

```rust
impl<T> Vec0<T> {
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.ptr, self.len)
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.ptr, self.len)
        }
    }
}
```

Now we can use slice methods:

```rust
let mut vec = Vec0::new();
vec.push(1);
vec.push(2);
vec.push(3);

let slice = vec.as_slice();
slice.len()      // 3
slice[0]         // 1
slice.iter()     // Iterator over &T
```

### Deref Coercion

Make `Vec0<T>` deref to `[T]`:

```rust
use std::ops::{Deref, DerefMut};

impl<T> Deref for Vec0<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for Vec0<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}
```

Now we can call slice methods directly:

```rust
let mut vec = Vec0::new();
vec.push(3);
vec.push(1);
vec.push(2);

vec.sort();       // Calls [T]::sort()
vec.len()         // Works! (both Vec and slice have len())
vec.iter()        // Calls [T]::iter()
```

## String is Just Vec<u8>

`String` is literally:

```rust
pub struct String {
    vec: Vec<u8>,
}
```

All String methods delegate to Vec:

```rust
impl String {
    pub fn new() -> String {
        String { vec: Vec::new() }
    }

    pub fn push_str(&mut self, s: &str) {
        self.vec.extend_from_slice(s.as_bytes());
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.vec)
        }
    }
}
```

### str is a Slice

`&str` is to `String` what `&[T]` is to `Vec<T>`:

```
String          &str
Vec<u8>         &[u8]  (but guaranteed valid UTF-8)
```

Both are fat pointers:

```rust
&str = (ptr: *const u8, len: usize)
```

```rust
let s = String::from("hello");
let slice: &str = &s[0..3];  // "hel"
```

## Memory Layout Comparison

### Array: Stack

```
[1, 2, 3]

Stack:
┌────┬────┬────┐
│ 1  │ 2  │ 3  │
└────┴────┴────┘
```

### Box: Heap (single value)

```
Box::new([1, 2, 3])

Stack:           Heap:
┌─────┐         ┌───┬───┬───┐
│ ptr │────────>│ 1 │ 2 │ 3 │
└─────┘         └───┴───┴───┘
```

### Vec: Heap (growable)

```
let mut vec = Vec::new();
vec.push(1);
vec.push(2);
vec.push(3);

Stack:                  Heap:
┌─────┬─────┬─────┐   ┌────┬────┬────┬────┬────┬────┐
│ ptr │ len │ cap │   │ 1  │ 2  │ 3  │ ?  │ ?  │ ?  │
│  •  │  3  │  6  │   └────┴────┴────┴────┴────┴────┘
└──│──┴─────┴─────┘    ^
   └───────────────────┘
```

`Vec` on stack: 24 bytes (on 64-bit: 8 + 8 + 8)
Actual data: on heap

### Slice: View (no ownership)

```
let vec = vec![1, 2, 3, 4, 5];
let slice = &vec[1..4];  // [2, 3, 4]

Stack (vec):            Heap:
┌─────┬─────┬─────┐   ┌───┬───┬───┬───┬───┐
│ ptr │ len │ cap │   │ 1 │ 2 │ 3 │ 4 │ 5 │
│  •  │  5  │  5  │   └───┴───┴───┴───┴───┘
└──│──┴─────┴─────┘         ^
   └────────────────────────┤
Stack (slice):              │
┌─────┬─────┐               │
│ ptr │ len │               │
│  •  │  3  │               │
└──│──┴─────┘               │
   └────────────────────────┘ (points to 2nd element)
```

## Common Operations

### Creating a Vec

```rust
let vec = Vec0::new();
// ptr = dangling, len = 0, capacity = 0

let mut vec = Vec0::new();
vec.push(1);
// First allocation: capacity = 1

vec.push(2);
// Grows: capacity = 2

vec.push(3);
// Grows: capacity = 4
```

### Preallocating Capacity

```rust
impl<T> Vec0<T> {
    pub fn with_capacity(capacity: usize) -> Vec0<T> {
        if capacity == 0 {
            return Vec0::new();
        }

        let layout = Layout::array::<T>(capacity).unwrap();
        let ptr = unsafe { alloc(layout) as *mut T };

        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }

        Vec0 {
            ptr,
            len: 0,
            capacity,
        }
    }
}
```

Use when you know the size upfront:

```rust
let mut vec = Vec0::with_capacity(100);
// 100 pushes without reallocation
for i in 0..100 {
    vec.push(i);
}
```

### Clear

```rust
impl<T> Vec0<T> {
    pub fn clear(&mut self) {
        // Drop all elements
        unsafe {
            ptr::drop_in_place(
                std::slice::from_raw_parts_mut(self.ptr, self.len)
            );
        }
        self.len = 0;
        // Capacity unchanged
    }
}
```

## Key Differences: Box vs Vec

| Feature  | Box\<T\>           | Vec\<T\>             |
| -------- | ------------------ | -------------------- |
| Size     | Fixed (one T)      | Dynamic              |
| Capacity | Always equals size | Can exceed size      |
| Growth   | N/A                | Doubles when full    |
| Use case | Single heap value  | Collection of values |
| Deref    | To `T`             | To `[T]`             |

## Performance Characteristics

| Operation      | Time Complexity | Notes                   |
| -------------- | --------------- | ----------------------- |
| `push`         | O(1) amortized  | O(n) on reallocation    |
| `pop`          | O(1)            | No reallocation         |
| `index`        | O(1)            | Direct memory access    |
| `insert(0, x)` | O(n)            | Must shift all elements |
| `remove(i)`    | O(n)            | Must shift elements     |

## The Complete Implementation

See `examples/04_vec.rs` for the full implementation with:

- `push`, `pop`, `insert`, `remove`
- `Index` and `IndexMut`
- `Deref` to `[T]`
- `IntoIterator` implementation
- `Clone` for `T: Clone`
- `Debug` for `T: Debug`

## Key Takeaways

1. **Vec uses raw allocator APIs** - Not implemented with `Box`
2. **Three fields** - `ptr`, `len`, `capacity`
3. **Growth strategy** - Double capacity when full
4. **String = Vec\<u8\>** - Same structure, UTF-8 constraint
5. **Slices are views** - `&[T]` and `&str` don't own data
6. **Fat pointers** - Slices contain `(ptr, len)`

## Exercises

See ./examples/04_vec.rs for exercises.

### Implement a slice-like type

Here's a starting point for a slice-like type:

```rust
use std::marker::PhantomData;

pub struct MySlice<'a, T> {
    ptr: *const T,
    len: usize,
    _marker: PhantomData<&'a T>,  // Zero-sized, but tells compiler about 'a and T
}

impl<'a, T> MySlice<'a, T> {
    pub fn from_vec(vec: &'a Vec0<T>) -> MySlice<'a, T> {
        MySlice {
            ptr: vec.ptr,
            len: vec.len,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, index: usize) -> Option<&'a T> {
        if index < self.len {
            unsafe { Some(&*self.ptr.add(index)) }
        } else {
            None
        }
    }
}
```

**Why PhantomData?**

Raw pointers (`*const T` and `*mut T`) don't carry lifetime information. Without `PhantomData`, the compiler wouldn't know that `MySlice<'a, T>` should:

1. **Not outlive the data it points to** - The `'a` lifetime connects the slice to the vec
2. **Act like it owns a `&'a T`** - For variance and drop check purposes

Example of what could go wrong without it:

```rust
// WITHOUT PhantomData, this dangerous code might compile:
let slice = {
    let vec = Vec0::new();
    vec.push(42);
    MySlice::from_vec(&vec)  // vec dies here!
}; // slice now points to freed memory! ❌ Use-after-free!

// WITH PhantomData, the compiler catches this:
// error: `vec` does not live long enough
```

`PhantomData<&'a T>` is **zero-sized** (no runtime cost) but tells the compiler: "pretend I own a `&'a T` reference" so it enforces proper lifetimes. With it, the above code becomes a **compile-time error** instead of undefined behavior.

**Alternative without PhantomData:**

You could use real references instead of raw pointers:

```rust
pub struct MySlice<'a, T> {
    data: &'a [T],  // Real reference, carries lifetime automatically
}
```

But this defeats the purpose of the exercise - we want to see what we can build with just `(ptr, len)`!

**What you can implement:**

- Index access (`impl Index<usize>`)
- `len()`, `is_empty()`
- `first()`, `last()`
- `iter()` returning an iterator

**What you CANNOT implement:**

- Slice syntax: `&my_slice[1..3]` (requires compiler support)
- Pattern matching: `match my_slice { [first, rest @ ..] => ... }` (DST feature)
- Automatic coercion from arrays: `&[1, 2, 3]` → `MySlice` (compiler magic)

This demonstrates why slices are special - they need compiler integration for the syntax we take for granted!

## Next Chapter

[Cell](./05-cell.md) - Interior mutability for `Copy` types.
