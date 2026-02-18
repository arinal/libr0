//! Vec0 - Educational reimplementation of `Vec<T>`

//! ```
//! use rustlib::vec::Vec0;
//! #[macro_use]
//! extern crate rustlib;
//! ```

use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr;

pub struct Vec0<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
}

impl<T> Vec0<T> {
    /// Creates an empty vector without allocating.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let v: Vec0<i32> = Vec0::new();
    /// assert_eq!(v.len(), 0);
    /// assert_eq!(v.capacity(), 0);
    /// ```
    pub fn new() -> Vec0<T> {
        Vec0 {
            ptr: std::ptr::NonNull::dangling().as_ptr(),
            len: 0,
            capacity: 0,
        }
    }

    /// Creates an empty vector with preallocated capacity.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let v: Vec0<i32> = Vec0::with_capacity(10);
    /// assert_eq!(v.len(), 0);
    /// assert_eq!(v.capacity(), 10);
    /// ```
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

    /// Returns the number of elements in the vector.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let mut v = Vec0::new();
    /// v.push(1);
    /// assert_eq!(v.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the total capacity (allocated space).
    /// ```
    /// use rustlib::vec::Vec0;
    /// let v: Vec0<i32> = Vec0::with_capacity(10);
    /// assert_eq!(v.capacity(), 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns `true` if the vector contains no elements.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let v: Vec0<i32> = Vec0::new();
    /// assert!(v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends an element to the end of the vector.
    /// Grows capacity if needed (doubles each time).
    /// ```
    /// use rustlib::vec::Vec0;
    /// let mut v = Vec0::new();
    /// v.push(1);
    /// v.push(2);
    /// assert_eq!(v.len(), 2);
    /// ```
    pub fn push(&mut self, value: T) {
        self.grow_if_needed();

        unsafe {
            ptr::write(self.ptr.add(self.len), value);
        }
        self.len += 1;
    }

    /// Removes and returns the last element, or [`None`] if empty.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let mut v = Vec0::new();
    /// v.push(1);
    /// assert_eq!(v.pop(), Some(1));
    /// assert_eq!(v.pop(), None);
    /// ```
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        unsafe { Some(ptr::read(self.ptr.add(self.len))) }
    }

    /// Inserts an element at position `index`, shifting elements to the right.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let mut v = Vec0::new();
    /// v.push(1);
    /// v.push(3);
    /// v.insert(1, 2);
    /// // v == [1, 2, 3]
    /// ```
    pub fn insert(&mut self, index: usize, value: T) {
        if index > self.len {
            panic!("insert index out of bounds: {} > {}", index, self.len);
        }

        self.grow_if_needed();

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

    /// Removes and returns the element at position `index`, shifting elements to the left.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let mut v = Vec0::new();
    /// v.push(1);
    /// v.push(2);
    /// v.push(3);
    /// assert_eq!(v.remove(1), 2);
    /// // v == [1, 3]
    /// ```
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

    /// Clears the vector, removing all elements. Capacity remains unchanged.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let mut v = Vec0::new();
    /// v.push(1);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    pub fn clear(&mut self) {
        if self.len > 0 {
            unsafe {
                ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(self.ptr, self.len));
            }
            self.len = 0;
        }
    }

    /// Shrinks the capacity to match the length.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let mut v = Vec0::with_capacity(10);
    /// v.push(1);
    /// v.shrink_to_fit();
    /// assert_eq!(v.capacity(), 1);
    /// ```
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

        let new_ptr =
            unsafe { realloc(self.ptr as *mut u8, old_layout, new_layout.size()) as *mut T };

        if new_ptr.is_null() {
            std::alloc::handle_alloc_error(new_layout);
        }

        self.ptr = new_ptr;
        self.capacity = self.len;
    }

    /// Returns a reference to the elements as a slice.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let mut v = Vec0::new();
    /// v.push(1);
    /// v.push(2);
    /// let slice = v.as_slice();
    /// assert_eq!(slice[0], 1);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Returns a mutable reference to the elements as a slice.
    /// ```
    /// use rustlib::vec::Vec0;
    /// let mut v = Vec0::new();
    /// v.push(1);
    /// v.as_mut_slice()[0] = 2;
    /// assert_eq!(v[0], 2);
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }

    fn grow_if_needed(&mut self) {
        if self.len == self.capacity {
            self.grow();
        }
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
            unsafe { realloc(self.ptr as *mut u8, old_layout, new_layout.size()) as *mut T }
        };

        if new_ptr.is_null() {
            std::alloc::handle_alloc_error(new_layout);
        }

        self.ptr = new_ptr;
        self.capacity = new_capacity;
    }
}

impl<T> Default for Vec0<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Indexing into [`Vec0`] returns a reference to the element.
/// ```
/// use rustlib::vec::Vec0;
/// let mut v = Vec0::new();
/// v.push(10);
/// assert_eq!(v[0], 10);
/// ```
impl<T> Index<usize> for Vec0<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        if index >= self.len {
            panic!("index out of bounds: {} >= {}", index, self.len);
        }
        unsafe { &*self.ptr.add(index) }
    }
}

/// Mutable indexing allows modifying elements.
/// ```
/// use rustlib::vec::Vec0;
/// let mut v = Vec0::new();
/// v.push(10);
/// v[0] = 20;
/// assert_eq!(v[0], 20);
/// ```
impl<T> IndexMut<usize> for Vec0<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        if index >= self.len {
            panic!("index out of bounds: {} >= {}", index, self.len);
        }
        unsafe { &mut *self.ptr.add(index) }
    }
}

/// Dropping a [`Vec0`] drops all elements and deallocates memory.
/// ```
/// use rustlib::vec::Vec0;
/// {
///     let mut v = Vec0::new();
///     v.push(String::from("hello"));
/// } // v dropped here, memory freed
/// ```
impl<T> Drop for Vec0<T> {
    fn drop(&mut self) {
        if self.capacity > 0 {
            unsafe {
                ptr::drop_in_place(std::ptr::slice_from_raw_parts_mut(self.ptr, self.len));
                let layout = Layout::array::<T>(self.capacity).unwrap();
                dealloc(self.ptr as *mut u8, layout);
            }
        }
    }
}

/// Dereferencing a [`Vec0<T>`] yields a `&[T]` slice.
/// ```
/// use rustlib::vec::Vec0;
/// let mut v = Vec0::new();
/// v.push(1);
/// v.push(2);
/// let _iter = v.iter(); // Uses [T]::iter() via deref coercion
/// ```
impl<T> Deref for Vec0<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

/// Mutable dereferencing yields a `&mut [T]` slice.
/// ```
/// use rustlib::vec::Vec0;
/// let mut v = Vec0::new();
/// v.push(3);
/// v.push(1);
/// v.push(2);
/// v.sort(); // Uses [T]::sort() via deref coercion
/// assert_eq!(v[0], 1);
/// assert_eq!(v[1], 2);
/// assert_eq!(v[2], 3);
/// ```
impl<T> DerefMut for Vec0<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

/// Cloning creates a new [`Vec0`] with deep-copied elements.
/// ```
/// use rustlib::vec::Vec0;
/// let mut v1 = Vec0::new();
/// v1.push(1);
/// let v2 = v1.clone();
/// assert_eq!(v1[0], 1);
/// assert_eq!(v2[0], 1); // independent copy
/// ```
impl<T: Clone> Clone for Vec0<T> {
    fn clone(&self) -> Vec0<T> {
        let mut new_vec = Vec0::with_capacity(self.len);
        for i in 0..self.len {
            new_vec.push(self[i].clone());
        }
        new_vec
    }
}

/// Debug formatting shows the vector as a list.
/// ```
/// use rustlib::vec::Vec0;
/// let mut v = Vec0::new();
/// v.push(1);
/// v.push(2);
/// assert_eq!(format!("{:?}", v), "[1, 2]");
/// ```
impl<T: std::fmt::Debug> std::fmt::Debug for Vec0<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

// ============================================================================
// IntoIterator implementation
// ============================================================================

/// Iterator that consumes a [`Vec0`] and yields owned elements.
/// Created by calling [`Vec0::into_iter`].
pub struct IntoIter<T> {
    ptr: *mut T,
    len: usize,
    capacity: usize,
    index: usize,
}

/// Iterating over [`IntoIter`] yields owned elements.
/// ```
/// use rustlib::vec0;
/// let v = vec0![1, 2, 3];
/// let mut iter = v.into_iter();
/// assert_eq!(iter.next(), Some(1));
/// assert_eq!(iter.next(), Some(2));
/// ```
impl<T> Iterator for IntoIter<T> {
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

/// Dropping [`IntoIter`] drops remaining unconsumed elements and frees memory.
/// ```
/// use rustlib::vec0;
/// let v = vec0![String::from("a"), String::from("b")];
/// let mut iter = v.into_iter();
/// assert_eq!(iter.next(), Some(String::from("a")));
/// // iter dropped, "b" is dropped and memory freed
/// ```
impl<T> Drop for IntoIter<T> {
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

/// Converting [`Vec0`] into an iterator yields owned elements.
/// ```
/// use rustlib::vec0;
/// let v = vec0![1, 2, 3];
/// let mut sum = 0;
/// for val in v {
///     sum += val; // Takes ownership of each element
/// }
/// assert_eq!(sum, 6);
/// // v is consumed, can't be used anymore
/// ```
impl<T> IntoIterator for Vec0<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let iter = IntoIter {
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
// vec! macro - syntactic sugar for creating vectors
// ============================================================================

/// Creates a [`Vec0`] containing the given elements.
///
/// ```
/// use rustlib::vec0;
/// use rustlib::vec::Vec0;
/// // Empty vector
/// let v: Vec0<i32> = vec0![];
///
/// // Vector with elements
/// let v = vec0![1, 2, 3];
///
/// // Vector with n copies of an element
/// let v = vec0![0; 5]; // [0, 0, 0, 0, 0]
/// ```
#[macro_export]
macro_rules! vec0 {
    () => {
        $crate::Vec0::new()
    };
    ($elem:expr; $n:expr) => {{
        let mut v = $crate::Vec0::with_capacity($n);
        #[allow(clippy::reversed_empty_ranges)]
        for _ in 0..$n {
            v.push($elem.clone());
        }
        v
    }};
    ($($x:expr),+ $(,)?) => {{
        let mut v = $crate::Vec0::new();
        $(v.push($x);)*
        v
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let vec: Vec0<i32> = Vec0::new();
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_with_capacity() {
        let vec: Vec0<i32> = Vec0::with_capacity(10);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.capacity(), 10);
    }

    #[test]
    fn test_push_and_pop() {
        let mut vec = Vec0::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.len(), 3);
        assert_eq!(vec.pop(), Some(3));
        assert_eq!(vec.pop(), Some(2));
        assert_eq!(vec.pop(), Some(1));
        assert_eq!(vec.pop(), None);
    }

    #[test]
    fn test_growth() {
        let mut vec = Vec0::new();
        assert_eq!(vec.capacity(), 0);

        vec.push(1);
        assert_eq!(vec.capacity(), 1);

        vec.push(2);
        assert_eq!(vec.capacity(), 2);

        vec.push(3);
        assert_eq!(vec.capacity(), 4);
    }

    #[test]
    fn test_index() {
        let mut vec = Vec0::new();
        vec.push(10);
        vec.push(20);
        vec.push(30);

        assert_eq!(vec[0], 10);
        assert_eq!(vec[1], 20);
        assert_eq!(vec[2], 30);
    }

    #[test]
    fn test_index_mut() {
        let mut vec = Vec0::new();
        vec.push(10);
        vec.push(20);

        vec[0] = 100;
        assert_eq!(vec[0], 100);
    }

    #[test]
    #[should_panic(expected = "index out of bounds")]
    fn test_index_out_of_bounds() {
        let vec: Vec0<i32> = Vec0::new();
        let _ = vec[0];
    }

    #[test]
    fn test_insert() {
        let mut vec = Vec0::new();
        vec.push(1);
        vec.push(3);
        vec.insert(1, 2);

        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 2);
        assert_eq!(vec[2], 3);
    }

    #[test]
    fn test_remove() {
        let mut vec = Vec0::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        assert_eq!(vec.remove(1), 2);
        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 1);
        assert_eq!(vec[1], 3);
    }

    #[test]
    fn test_clear() {
        let mut vec = Vec0::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        vec.clear();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
    }

    #[test]
    fn test_shrink_to_fit() {
        let mut vec = Vec0::with_capacity(10);
        vec.push(1);
        vec.push(2);

        assert_eq!(vec.capacity(), 10);
        vec.shrink_to_fit();
        assert_eq!(vec.capacity(), 2);
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_deref_to_slice() {
        let mut vec = Vec0::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let slice: &[i32] = &vec;
        assert_eq!(slice.len(), 3);
        assert_eq!(slice[0], 1);
    }

    #[test]
    fn test_into_iter() {
        let mut vec = Vec0::new();
        vec.push(1);
        vec.push(2);
        vec.push(3);

        let mut sum = 0;
        for val in vec {
            sum += val;
        }
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_clone() {
        let mut vec = Vec0::new();
        vec.push(1);
        vec.push(2);

        let vec2 = vec.clone();
        assert_eq!(vec[0], vec2[0]);
        assert_eq!(vec[1], vec2[1]);
    }

    #[test]
    fn test_debug() {
        let mut vec = Vec0::new();
        vec.push(1);
        vec.push(2);

        assert_eq!(format!("{:?}", vec), "[1, 2]");
    }

    #[test]
    fn test_drop_elements() {
        use std::sync::Arc;

        let item = Arc::new(42);
        assert_eq!(Arc::strong_count(&item), 1);

        {
            let mut vec = Vec0::new();
            vec.push(item.clone());
            vec.push(item.clone());
            assert_eq!(Arc::strong_count(&item), 3);
        }
        // Vec dropped, items should be dropped
        assert_eq!(Arc::strong_count(&item), 1);
    }

    #[test]
    fn test_my_vec_macro_empty() {
        let v: Vec0<i32> = vec0![];
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_my_vec_macro_elements() {
        let v = vec0![1, 2, 3];
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn test_my_vec_macro_repeat() {
        let v = vec0![0; 5];
        assert_eq!(v.len(), 5);
        for i in 0..5 {
            assert_eq!(v[i], 0);
        }
    }

    #[test]
    fn test_my_vec_macro_single_element() {
        let v = vec0![42];
        assert_eq!(v.len(), 1);
        assert_eq!(v[0], 42);
    }

    #[test]
    fn test_my_vec_macro_trailing_comma() {
        let v = vec0![1, 2, 3,];
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 2);
        assert_eq!(v[2], 3);
    }

    #[test]
    fn test_my_vec_macro_strings() {
        let v = vec0![String::from("hello"), String::from("world")];
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], "hello");
        assert_eq!(v[1], "world");
    }

    #[test]
    fn test_my_vec_macro_repeat_string() {
        let v = vec0![String::from("test"); 3];
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], "test");
        assert_eq!(v[1], "test");
        assert_eq!(v[2], "test");
    }

    #[test]
    fn test_my_vec_macro_expressions() {
        let v = vec0![1 + 1, 2 * 2, 3 - 1];
        assert_eq!(v.len(), 3);
        assert_eq!(v[0], 2);
        assert_eq!(v[1], 4);
        assert_eq!(v[2], 2);
    }

    #[test]
    fn test_my_vec_macro_repeat_zero() {
        let v: Vec0<i32> = vec0![42; 0];
        assert_eq!(v.len(), 0);
        assert!(v.is_empty());
    }
}

