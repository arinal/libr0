//! Box0 - Educational reimplementation of `Box<T>`

use std::alloc::{alloc, dealloc, Layout};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::ptr;

pub struct Box0<T> {
    ptr: *mut T,
}

impl<T> Box0<T> {
    /// Allocates memory on the heap and places `value` into it.
    /// ```
    /// use rustlib::r#box::Box0;
    /// let b = Box0::new(42);
    /// assert_eq!(*b, 42);
    /// ```
    pub fn new(value: T) -> Box0<T> {
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

            Box0 { ptr }
        }
    }

    /// Consumes the [`Box0`], returning the wrapped value.
    /// ```
    /// use rustlib::r#box::Box0;
    /// let b = Box0::new(42);
    /// assert_eq!(b.into_inner(), 42); // b no longer exists
    /// ```
    pub fn into_inner(self) -> T {
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

    /// Consumes and leaks the [`Box0`], returning a mutable reference with `'static` lifetime.
    /// The memory is never freed.
    /// ```
    /// use rustlib::r#box::Box0;
    /// let b = Box0::new(42);
    /// let leaked: &'static mut i32 = b.leak();
    /// *leaked = 100;
    /// ```
    pub fn leak(self) -> &'static mut T {
        let ptr = self.ptr;
        std::mem::forget(self); // Don't run Drop
        unsafe { &mut *ptr }
    }

    /// Consumes the [`Box0`], returning a raw pointer.
    /// The caller is responsible for the memory.
    /// ```
    /// use rustlib::r#box::Box0;
    /// let b = Box0::new(42);
    /// let ptr = Box0::into_raw(b);
    /// ```
    pub fn into_raw(self) -> *mut T {
        let ptr = self.ptr;
        std::mem::forget(self); // Don't run Drop
        ptr
    }

    /// Constructs a [`Box0`] from a raw pointer.
    /// ```
    /// use rustlib::r#box::Box0;
    /// let b = Box0::new(42);
    /// let ptr = Box0::into_raw(b);
    /// unsafe { Box0::from_raw(ptr) }; // Box0(42)
    /// ```
    ///
    /// # Safety
    ///
    /// This function is unsafe because improper use may lead to memory problems:
    /// - The pointer must have been previously returned by [`Box0::into_raw`]
    /// - After calling this function, the raw pointer is owned by the resulting [`Box0`].
    ///   Do not use the pointer again or call `from_raw` twice with the same pointer,
    ///   as this will cause a double-free.
    pub unsafe fn from_raw(ptr: *mut T) -> Box0<T> {
        Box0 { ptr }
    }

    /// Maps a [`Box0<T>`] to [`Box0<U>`] by applying a function to the contained value.
    /// ```
    /// use rustlib::r#box::Box0;
    /// Box0::new(5).map(|x| x * 2); // Box0(10)
    /// Box0::new("hello").map(|s| s.len()); // Box0(5)
    /// ```
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Box0<U> {
        let value = self.into_inner();
        Box0::new(f(value))
    }
}

/// Dereferencing a [`Box0<T>`] yields a reference to `T`.
/// ```
/// use rustlib::r#box::Box0;
/// let b = Box0::new(42);
/// assert_eq!(*b, 42);
/// ```
impl<T> Deref for Box0<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

/// Mutable dereferencing allows modifying the contained value.
/// ```
/// use rustlib::r#box::Box0;
/// let mut b = Box0::new(42);
/// *b = 100;
/// assert_eq!(*b, 100);
/// ```
impl<T> DerefMut for Box0<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

/// Dropping a [`Box0<T>`] runs the destructor for `T` and frees the heap memory.
/// ```
/// use rustlib::r#box::Box0;
/// {
///     let b = Box0::new(String::from("hello"));
/// } // b dropped here, memory freed
/// ```
impl<T> Drop for Box0<T> {
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

/// Debug formatting shows the contained value.
/// ```
/// use rustlib::r#box::Box0;
/// let b = Box0::new(42);
/// format!("{:?}", b); // "Box0(42)"
/// ```
impl<T: fmt::Debug> fmt::Debug for Box0<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Box0").field(&**self).finish()
    }
}

/// Cloning creates a new [`Box0`] with a deep copy of the value.
/// ```
/// use rustlib::r#box::Box0;
/// let b1 = Box0::new(42);
/// let b2 = b1.clone();
/// assert_eq!(*b1, 42);
/// assert_eq!(*b2, 42); // independent copy
/// ```
impl<T: Clone> Clone for Box0<T> {
    fn clone(&self) -> Self {
        Box0::new((**self).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_deref() {
        let boxed = Box0::new(42);
        assert_eq!(*boxed, 42);
    }

    #[test]
    fn test_deref_mut() {
        let mut boxed = Box0::new(42);
        *boxed = 100;
        assert_eq!(*boxed, 100);
    }

    #[test]
    fn test_into_inner() {
        let boxed = Box0::new(42);
        let value = boxed.into_inner();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_map() {
        let boxed = Box0::new(10);
        let mapped = boxed.map(|x| x * 2);
        assert_eq!(*mapped, 20);
    }

    #[test]
    fn test_into_raw_and_from_raw() {
        let boxed = Box0::new(42);
        let raw = boxed.into_raw();

        unsafe {
            let restored = Box0::from_raw(raw);
            assert_eq!(*restored, 42);
        }
    }

    #[test]
    fn test_leak() {
        let boxed = Box0::new(42);
        let leaked: &'static mut i32 = boxed.leak();
        assert_eq!(*leaked, 42);

        // Can modify leaked value
        *leaked = 100;
        assert_eq!(*leaked, 100);

        // Clean up manually (normally this would leak forever)
        unsafe {
            let ptr = leaked as *mut i32;
            let layout = Layout::new::<i32>();
            ptr::drop_in_place(ptr);
            dealloc(ptr as *mut u8, layout);
        }
    }

    #[test]
    fn test_clone() {
        let boxed1 = Box0::new(42);
        let boxed2 = boxed1.clone();

        assert_eq!(*boxed1, *boxed2);

        // They're independent
        drop(boxed1);
        assert_eq!(*boxed2, 42);
    }

    #[test]
    fn test_debug() {
        let boxed = Box0::new(42);
        assert_eq!(format!("{:?}", boxed), "Box0(42)");
    }

    #[test]
    fn test_with_string() {
        let boxed = Box0::new(String::from("hello"));
        assert_eq!(*boxed, "hello");
        assert_eq!(boxed.len(), 5);
    }

    #[test]
    fn test_drop() {
        use std::sync::Arc;

        let drop_checker = Arc::new(());
        assert_eq!(Arc::strong_count(&drop_checker), 1);

        {
            let _boxed = Box0::new(drop_checker.clone());
            assert_eq!(Arc::strong_count(&drop_checker), 2);
        }

        // Box dropped, Arc count should be back to 1
        assert_eq!(Arc::strong_count(&drop_checker), 1);
    }

    #[test]
    fn test_deref_coercion() {
        let boxed = Box0::new(String::from("hello"));

        // Should work with functions that take &str
        fn take_str(s: &str) -> usize {
            s.len()
        }

        assert_eq!(take_str(&boxed), 5);
    }

    #[test]
    fn test_nested_box() {
        let boxed = Box0::new(Box0::new(42));
        assert_eq!(**boxed, 42);
    }
}
