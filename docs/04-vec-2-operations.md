# Operations & Performance

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

**Complete solutions:** Switch to the `answers` branch with `git checkout answers` to see completed exercises

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
