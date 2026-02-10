//! MyCell - Educational reimplementation of `Cell<T>`

use std::cell::UnsafeCell;

/// A mutable memory location with interior mutability.
/// Allows mutation through shared references without borrowing rules.
/// Only works in single-threaded contexts (!Sync).
pub struct MyCell<T> {
    value: UnsafeCell<T>,
}

// Cell is !Sync - can't be shared between threads
// This is automatically inferred from UnsafeCell

impl<T> MyCell<T> {
    /// Creates a new cell containing the given value.
    /// ```
    /// use rustlib::cell::MyCell;
    /// let cell = MyCell::new(42);
    /// assert_eq!(cell.get(), 42);
    /// ```
    pub fn new(value: T) -> MyCell<T> {
        MyCell {
            value: UnsafeCell::new(value),
        }
    }

    /// Sets the contained value.
    /// ```
    /// use rustlib::cell::MyCell;
    /// let cell = MyCell::new(10);
    /// cell.set(20);
    /// assert_eq!(cell.get(), 20);
    /// ```
    pub fn set(&self, value: T) {
        // SAFETY: Single-threaded, no references escape
        unsafe {
            *self.value.get() = value;
        }
    }

    /// Replaces the contained value and returns the old value.
    /// ```
    /// use rustlib::cell::MyCell;
    /// let cell = MyCell::new(10);
    /// let old = cell.replace(20);
    /// assert_eq!(old, 10);
    /// assert_eq!(cell.get(), 20);
    /// ```
    pub fn replace(&self, value: T) -> T {
        // SAFETY: Single-threaded, no references escape
        unsafe { std::mem::replace(&mut *self.value.get(), value) }
    }

    /// Consumes the cell and returns the contained value.
    /// ```
    /// use rustlib::cell::MyCell;
    /// let cell = MyCell::new(42);
    /// assert_eq!(cell.into_inner(), 42);
    /// ```
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    /// Returns a raw pointer to the underlying data.
    /// ```
    /// use rustlib::cell::MyCell;
    /// let cell = MyCell::new(42);
    /// let ptr = cell.as_ptr();
    /// assert_eq!(unsafe { *ptr }, 42);
    /// ```
    pub fn as_ptr(&self) -> *mut T {
        self.value.get()
    }

    /// Swaps the values of two cells.
    /// ```
    /// use rustlib::cell::MyCell;
    /// let a = MyCell::new(10);
    /// let b = MyCell::new(20);
    /// a.swap(&b);
    /// assert_eq!(a.get(), 20);
    /// assert_eq!(b.get(), 10);
    /// ```
    pub fn swap(&self, other: &MyCell<T>) {
        // SAFETY: Single-threaded, no references escape
        unsafe {
            std::ptr::swap(self.value.get(), other.value.get());
        }
    }
}

impl<T: Copy> MyCell<T> {
    /// Returns a copy of the contained value.
    /// Only available for types that implement [`Copy`].
    /// ```
    /// use rustlib::cell::MyCell;
    /// let cell = MyCell::new(42);
    /// assert_eq!(cell.get(), 42);
    /// ```
    pub fn get(&self) -> T {
        // SAFETY: We only copy out, never expose a reference
        unsafe { *self.value.get() }
    }

    /// Updates the contained value using the provided function.
    /// ```
    /// use rustlib::cell::MyCell;
    /// let cell = MyCell::new(10);
    /// cell.update(|x| x * 2);
    /// assert_eq!(cell.get(), 20);
    /// ```
    pub fn update<F: FnOnce(T) -> T>(&self, f: F) {
        let old = self.get();
        self.set(f(old));
    }
}

impl<T: Default> MyCell<T> {
    /// Takes the value, replacing it with the default value.
    /// ```
    /// use rustlib::cell::MyCell;
    /// let cell = MyCell::new(Some(42));
    /// assert_eq!(cell.take(), Some(42));
    /// assert_eq!(cell.get(), None);
    /// ```
    pub fn take(&self) -> T {
        self.replace(T::default())
    }
}

/// Cloning a [`MyCell`] creates an independent copy.
/// ```
/// use rustlib::cell::MyCell;
/// let cell1 = MyCell::new(42);
/// let cell2 = cell1.clone();
/// cell1.set(100);
/// assert_eq!(cell1.get(), 100);
/// assert_eq!(cell2.get(), 42);
/// ```
impl<T: Clone> Clone for MyCell<T> {
    fn clone(&self) -> MyCell<T> {
        unsafe { MyCell::new((*self.value.get()).clone()) }
    }
}

/// Creates a cell with the default value.
/// ```
/// use rustlib::cell::MyCell;
/// let cell: MyCell<i32> = MyCell::default();
/// assert_eq!(cell.get(), 0);
/// ```
impl<T: Default> Default for MyCell<T> {
    fn default() -> MyCell<T> {
        MyCell::new(T::default())
    }
}

/// Debug formatting shows the contained value.
/// ```
/// use rustlib::cell::MyCell;
/// let cell = MyCell::new(42);
/// assert_eq!(format!("{:?}", cell), "MyCell(42)");
/// ```
impl<T: Copy + std::fmt::Debug> std::fmt::Debug for MyCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MyCell({:?})", self.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_get() {
        let cell = MyCell::new(42);
        assert_eq!(cell.get(), 42);
    }

    #[test]
    fn test_set() {
        let cell = MyCell::new(10);
        cell.set(20);
        assert_eq!(cell.get(), 20);
    }

    #[test]
    fn test_replace() {
        let cell = MyCell::new(10);
        let old = cell.replace(20);
        assert_eq!(old, 10);
        assert_eq!(cell.get(), 20);
    }

    #[test]
    fn test_swap() {
        let cell1 = MyCell::new(10);
        let cell2 = MyCell::new(20);

        cell1.swap(&cell2);

        assert_eq!(cell1.get(), 20);
        assert_eq!(cell2.get(), 10);
    }

    #[test]
    fn test_into_inner() {
        let cell = MyCell::new(42);
        assert_eq!(cell.into_inner(), 42);
    }

    #[test]
    fn test_take() {
        let cell = MyCell::new(Some(42));
        assert_eq!(cell.take(), Some(42));
        assert_eq!(cell.get(), None);
    }

    #[test]
    fn test_update() {
        let cell = MyCell::new(10);
        cell.update(|x| x * 2);
        assert_eq!(cell.get(), 20);
    }

    #[test]
    fn test_clone() {
        let cell = MyCell::new(42);
        let cell2 = cell.clone();
        assert_eq!(cell.get(), cell2.get());

        cell.set(100);
        assert_eq!(cell.get(), 100);
        assert_eq!(cell2.get(), 42); // Independent
    }

    #[test]
    fn test_default() {
        let cell: MyCell<i32> = MyCell::default();
        assert_eq!(cell.get(), 0);
    }

    #[test]
    fn test_debug() {
        let cell = MyCell::new(42);
        assert_eq!(format!("{:?}", cell), "MyCell(42)");
    }
}
