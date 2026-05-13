# Slices, Strings & Operations

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

```bob
Slice structure:
+---------------+-------------+
| ptr: *const T | len: usize  |
|      *        |             |
+------|--------+-------------+
       |  
       | "(points to array elements in memory)"
       |
       |       +---+---+---+
       +------>| T | T | T | 
               +---+---+---+
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

`[1, 2, 3]`

```bob
    Stack
+----+----+----+
| 1  | 2  | 3  |
+----+----+----+
```

### Box: Heap (single value)

**`Box::new([1, 2, 3])`**

```bob
Stack               Heap
+-------+         +---+---+---+
| ptr *-+-------->| 1 | 2 | 3 |
+-------+         +---+---+---+
```

### Vec: Heap (growable)

**After `vec.push(1); vec.push(2); vec.push(3); ... ; vec.push(7)`**

```bob
       STACK         |                HEAP
                     |
+-----------------+  |
| vec: Vec<i32>   |  |        <-------- len ---------->
+-----+-----+-----+  |      +---+---+---+---+---+---+---+---+---+---+
| "ptr:" *--------+--|----->| 1 | 2 | 3 | 4 | 5 | 6 | 7 | ? | ? | ? |
| "len:" 7        |  |      +---+---+---+---+---+---+---+---+---+---+
| "cap:" 10       |  |        <------------ capacity ------------->
+-----+-----+-----+  |
```

`Vec` on stack: 24 bytes (on 64-bit: 8 + 8 + 8)
Actual data: on heap

### Slice: View (no ownership)

**`let slice = &vec[1..5]; // [2, 3, 4, 5, 6]`**

```bob
       STACK         |                    HEAP 
                     |       
+-----------------+  | "index:"0   1   2   3   4   5   6   7   8   9
| vec: Vec<i32>   |  | "slice:"    1     "..."     5
+-----+-----+-----+  |       +---+---+---+---+---+---+---+---+---+---+
| "ptr:" *--------+--|------>| 1 | 2 | 3 | 4 | 5 | 6 | 7 | ? | ? | ? |
| "len:" 7        |  |       +---+-^-+---+---+---+---+---+---+---+---+
| "cap:" 10       |  |             |----- len ----->
+-----+-----+-----+  |             |
                     |             |        
                     |             |       
+---------------+    |             |
| slice: &[i32] |    |             |
+---------------+    |             |
| "ptr:" *------+----+-------------'
| "len:" 5      |    | 
+---------------+    | 
```                  
                    
