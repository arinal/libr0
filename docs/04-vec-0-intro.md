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

