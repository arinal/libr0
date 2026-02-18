//! Cell0 - Educational reimplementation of `Cell<T>`

use std::cell::UnsafeCell;

/// A mutable memory location with interior mutability.
/// Allows mutation through shared references without borrowing rules.
/// Only works in single-threaded contexts (!Sync).
pub struct Cell0<T: ?Sized> {
    value: UnsafeCell<T>,
}

// Cell is !Sync - can't be shared between threads
// This is automatically inferred from UnsafeCell

impl<T> Cell0<T> {
    /// Creates a new cell containing the given value.
    /// ```
    /// use rustlib::cell::Cell0;
    /// let cell = Cell0::new(42);
    /// assert_eq!(cell.get(), 42);
    /// ```
    pub fn new(value: T) -> Cell0<T> {
        Cell0 {
            value: UnsafeCell::new(value),
        }
    }

    /// Sets the contained value.
    /// ```
    /// use rustlib::cell::Cell0;
    /// let cell = Cell0::new(10);
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
    /// use rustlib::cell::Cell0;
    /// let cell = Cell0::new(10);
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
    /// use rustlib::cell::Cell0;
    /// let cell = Cell0::new(42);
    /// assert_eq!(cell.into_inner(), 42);
    /// ```
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    /// Returns a raw pointer to the underlying data.
    /// ```
    /// use rustlib::cell::Cell0;
    /// let cell = Cell0::new(42);
    /// let ptr = cell.as_ptr();
    /// assert_eq!(unsafe { *ptr }, 42);
    /// ```
    pub fn as_ptr(&self) -> *mut T {
        self.value.get()
    }

    /// Swaps the values of two cells.
    /// ```
    /// use rustlib::cell::Cell0;
    /// let a = Cell0::new(10);
    /// let b = Cell0::new(20);
    /// a.swap(&b);
    /// assert_eq!(a.get(), 20);
    /// assert_eq!(b.get(), 10);
    /// ```
    pub fn swap(&self, other: &Cell0<T>) {
        // SAFETY: Single-threaded, no references escape
        unsafe {
            std::ptr::swap(self.value.get(), other.value.get());
        }
    }
}

// Separate impl block with ?Sized to support dynamically sized types
impl<T: ?Sized> Cell0<T> {
    /// Returns a mutable reference when you have exclusive access to the Cell.
    ///
    /// Unlike other Cell methods that work with `&self`, this requires `&mut self`,
    /// giving you compile-time guaranteed exclusive access. This means you can safely
    /// get a real `&mut T` to the inner value, no copying needed.
    ///
    /// **Note:** This is rarely used in practice! Cell exists precisely so you DON'T
    /// need `&mut`. If you have `&mut Cell<T>`, you might as well have used `T` directly.
    /// For actual interior mutability through `&self`, use the other Cell methods like
    /// `set()` and `get()`, or consider `RefCell` if you need references to non-Copy types.
    ///
    /// **About `?Sized`:** This method uses `impl<T: ?Sized>` which removes the default
    /// `Sized` bound. This allows `get_mut` to work with dynamically sized types (DSTs)
    /// like `[T]` or `str`. Since this method works with references (`&mut T`), it doesn't
    /// need `T` to be `Sized` - references to DSTs are perfectly fine.
    ///
    /// ```
    /// use rustlib::cell::Cell0;
    ///
    /// let mut c = Cell0::new(5);
    /// *c.get_mut() += 1;  // Direct mutable access
    ///
    /// assert_eq!(c.get(), 6);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}

impl<T: Copy> Cell0<T> {
    /// Returns a copy of the contained value.
    /// Only available for types that implement [`Copy`].
    /// ```
    /// use rustlib::cell::Cell0;
    /// let cell = Cell0::new(42);
    /// assert_eq!(cell.get(), 42);
    /// ```
    pub fn get(&self) -> T {
        // SAFETY: We only copy out, never expose a reference
        unsafe { *self.value.get() }
    }

    /// Updates the contained value using the provided function.
    /// ```
    /// use rustlib::cell::Cell0;
    /// let cell = Cell0::new(10);
    /// cell.update(|x| x * 2);
    /// assert_eq!(cell.get(), 20);
    /// ```
    pub fn update<F: FnOnce(T) -> T>(&self, f: F) {
        let old = self.get();
        self.set(f(old));
    }
}

impl<T: Default> Cell0<T> {
    /// Takes the value, replacing it with the default value.
    /// ```
    /// use rustlib::cell::Cell0;
    /// let cell = Cell0::new(Some(42));
    /// assert_eq!(cell.take(), Some(42));
    /// assert_eq!(cell.get(), None);
    /// ```
    pub fn take(&self) -> T {
        self.replace(T::default())
    }
}

/// Cloning a [`Cell0`] creates an independent copy.
/// ```
/// use rustlib::cell::Cell0;
/// let cell1 = Cell0::new(42);
/// let cell2 = cell1.clone();
/// cell1.set(100);
/// assert_eq!(cell1.get(), 100);
/// assert_eq!(cell2.get(), 42);
/// ```
impl<T: Clone> Clone for Cell0<T> {
    fn clone(&self) -> Cell0<T> {
        unsafe { Cell0::new((*self.value.get()).clone()) }
    }
}

/// Creates a cell with the default value.
/// ```
/// use rustlib::cell::Cell0;
/// let cell: Cell0<i32> = Cell0::default();
/// assert_eq!(cell.get(), 0);
/// ```
impl<T: Default> Default for Cell0<T> {
    fn default() -> Cell0<T> {
        Cell0::new(T::default())
    }
}

/// Debug formatting shows the contained value.
/// ```
/// use rustlib::cell::Cell0;
/// let cell = Cell0::new(42);
/// assert_eq!(format!("{:?}", cell), "Cell0(42)");
/// ```
impl<T: Copy + std::fmt::Debug> std::fmt::Debug for Cell0<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cell0({:?})", self.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_get() {
        let cell = Cell0::new(42);
        assert_eq!(cell.get(), 42);
    }

    #[test]
    fn test_set() {
        let cell = Cell0::new(10);
        cell.set(20);
        assert_eq!(cell.get(), 20);
    }

    #[test]
    fn test_replace() {
        let cell = Cell0::new(10);
        let old = cell.replace(20);
        assert_eq!(old, 10);
        assert_eq!(cell.get(), 20);
    }

    #[test]
    fn test_swap() {
        let cell1 = Cell0::new(10);
        let cell2 = Cell0::new(20);

        cell1.swap(&cell2);

        assert_eq!(cell1.get(), 20);
        assert_eq!(cell2.get(), 10);
    }

    #[test]
    fn test_into_inner() {
        let cell = Cell0::new(42);
        assert_eq!(cell.into_inner(), 42);
    }

    #[test]
    fn test_take() {
        let cell = Cell0::new(Some(42));
        assert_eq!(cell.take(), Some(42));
        assert_eq!(cell.get(), None);
    }

    #[test]
    fn test_update() {
        let cell = Cell0::new(10);
        cell.update(|x| x * 2);
        assert_eq!(cell.get(), 20);
    }

    #[test]
    fn test_clone() {
        let cell = Cell0::new(42);
        let cell2 = cell.clone();
        assert_eq!(cell.get(), cell2.get());

        cell.set(100);
        assert_eq!(cell.get(), 100);
        assert_eq!(cell2.get(), 42); // Independent
    }

    #[test]
    fn test_default() {
        let cell: Cell0<i32> = Cell0::default();
        assert_eq!(cell.get(), 0);
    }

    #[test]
    fn test_debug() {
        let cell = Cell0::new(42);
        assert_eq!(format!("{:?}", cell), "Cell0(42)");
    }

    #[test]
    fn test_get_mut() {
        let mut cell = Cell0::new(5);
        *cell.get_mut() += 1;
        assert_eq!(cell.get(), 6);

        // Can also use get_mut to read
        let val_ref = cell.get_mut();
        assert_eq!(*val_ref, 6);
    }
}
