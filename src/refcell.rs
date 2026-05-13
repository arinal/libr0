//! RefCell0 - Educational reimplementation of `RefCell<T>`
//!
//! RefCell provides interior mutability through runtime borrow checking.
//! Unlike Cell, it works with any type by returning guard objects that
//! track borrows at runtime instead of compile time.

use std::cell::{Cell, UnsafeCell};
use std::ops::{Deref, DerefMut};

/// A mutable memory location with runtime borrow checking.
///
/// RefCell enforces Rust's borrowing rules at runtime instead of compile time:
/// - Multiple immutable borrows can coexist
/// - Only one mutable borrow at a time
/// - Can't have mutable and immutable borrows simultaneously
///
/// Violations cause panics instead of compile errors.
pub struct RefCell0<T> {
    /// Tracks borrow state:
    /// - 0 = not borrowed
    /// - positive = number of active immutable borrows
    /// - -1 = mutably borrowed
    borrow_count: Cell<isize>,
    /// The actual value, protected by runtime borrow checks
    value: UnsafeCell<T>,
}

/// An immutable guard providing access to borrowed data.
///
/// When dropped, automatically decrements the borrow count.
/// This is RAII (Resource Acquisition Is Initialization) in action.
pub struct Ref<'a, T> {
    refcell: &'a RefCell0<T>,
}

/// A mutable guard providing exclusive access to borrowed data.
///
/// When dropped, automatically resets the borrow count to 0.
pub struct RefMut<'a, T> {
    refcell: &'a RefCell0<T>,
}

/// Error returned when an immutable borrow fails.
#[derive(Debug)]
pub struct BorrowError;

/// Error returned when a mutable borrow fails.
#[derive(Debug)]
pub struct BorrowMutError;

impl<T> RefCell0<T> {
    /// Creates a new RefCell containing the given value.
    ///
    /// ```
    /// use rustlib::refcell::RefCell0;
    /// let cell = RefCell0::new(42);
    /// assert_eq!(*cell.borrow(), 42);
    /// ```
    pub fn new(value: T) -> RefCell0<T> {
        RefCell0 {
            borrow_count: Cell::new(0),
            value: UnsafeCell::new(value),
        }
    }

    /// Immutably borrows the wrapped value, panicking if already mutably borrowed.
    ///
    /// Multiple immutable borrows can coexist.
    ///
    /// ```
    /// use rustlib::refcell::RefCell0;
    /// let cell = RefCell0::new(5);
    ///
    /// let r1 = cell.borrow();
    /// let r2 = cell.borrow(); // OK: multiple immutable borrows
    /// assert_eq!(*r1, 5);
    /// assert_eq!(*r2, 5);
    /// ```
    ///
    /// # Panics
    /// Panics if the value is currently mutably borrowed.
    pub fn borrow(&self) -> Ref<'_, T> {
        self.try_borrow().expect("Already mutably borrowed")
    }

    /// Mutably borrows the wrapped value, panicking if already borrowed.
    ///
    /// ```
    /// use rustlib::refcell::RefCell0;
    /// let cell = RefCell0::new(5);
    ///
    /// {
    ///     let mut borrowed = cell.borrow_mut();
    ///     *borrowed += 1;
    /// } // mutable borrow ends here
    ///
    /// assert_eq!(*cell.borrow(), 6);
    /// ```
    ///
    /// # Panics
    /// Panics if the value is currently borrowed (mutably or immutably).
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.try_borrow_mut().expect("Already borrowed")
    }

    /// Immutably borrows the wrapped value, returning an error if already mutably borrowed.
    ///
    /// This is the non-panicking variant of [`borrow`](Self::borrow).
    ///
    /// ```
    /// use rustlib::refcell::RefCell0;
    /// let cell = RefCell0::new(5);
    ///
    /// {
    ///     let _guard = cell.borrow_mut();
    ///     // Can't borrow immutably while mutably borrowed
    ///     assert!(cell.try_borrow().is_err());
    /// }
    ///
    /// // Now we can borrow
    /// assert!(cell.try_borrow().is_ok());
    /// ```
    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        let count = self.borrow_count.get();
        // Can't borrow if mutably borrowed (count == -1)
        if count < 0 {
            Err(BorrowError)
        } else {
            // Increment count for this immutable borrow
            self.borrow_count.set(count + 1);
            Ok(Ref { refcell: self })
        }
    }

    /// Mutably borrows the wrapped value, returning an error if already borrowed.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](Self::borrow_mut).
    ///
    /// ```
    /// use rustlib::refcell::RefCell0;
    /// let cell = RefCell0::new(5);
    ///
    /// {
    ///     let _guard = cell.borrow();
    ///     // Can't borrow mutably while immutably borrowed
    ///     assert!(cell.try_borrow_mut().is_err());
    /// }
    ///
    /// // Now we can borrow mutably
    /// assert!(cell.try_borrow_mut().is_ok());
    /// ```
    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        // Can only mutably borrow if count is exactly 0
        if self.borrow_count.get() != 0 {
            Err(BorrowMutError)
        } else {
            // Set to -1 to indicate mutable borrow
            self.borrow_count.set(-1);
            Ok(RefMut { refcell: self })
        }
    }

    /// Consumes the RefCell and returns the wrapped value.
    ///
    /// ```
    /// use rustlib::refcell::RefCell0;
    /// let cell = RefCell0::new(42);
    /// assert_eq!(cell.into_inner(), 42);
    /// ```
    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    /// Replaces the wrapped value with a new one, returning the old value.
    ///
    /// ```
    /// use rustlib::refcell::RefCell0;
    /// let cell = RefCell0::new(42);
    /// let old = cell.replace(100);
    /// assert_eq!(old, 42);
    /// assert_eq!(*cell.borrow(), 100);
    /// ```
    ///
    /// # Panics
    /// Panics if the value is currently borrowed.
    pub fn replace(&self, value: T) -> T {
        // borrow_mut will panic if already borrowed
        std::mem::replace(&mut *self.borrow_mut(), value)
    }

    /// Swaps the wrapped value with another RefCell.
    ///
    /// ```
    /// use rustlib::refcell::RefCell0;
    /// let a = RefCell0::new(1);
    /// let b = RefCell0::new(2);
    ///
    /// a.swap(&b);
    ///
    /// assert_eq!(*a.borrow(), 2);
    /// assert_eq!(*b.borrow(), 1);
    /// ```
    ///
    /// # Panics
    /// Panics if either value is currently borrowed.
    pub fn swap(&self, other: &RefCell0<T>) {
        std::mem::swap(&mut *self.borrow_mut(), &mut *other.borrow_mut())
    }

    /// Returns a mutable reference when you have exclusive access.
    ///
    /// Unlike other RefCell methods that work with `&self`, this requires `&mut self`,
    /// giving you compile-time guaranteed exclusive access.
    ///
    /// **Note:** This is rarely used! RefCell exists precisely so you DON'T need `&mut`.
    /// If you have `&mut RefCell<T>`, you might as well have used `T` directly.
    ///
    /// ```
    /// use rustlib::refcell::RefCell0;
    /// let mut cell = RefCell0::new(5);
    /// *cell.get_mut() += 10;
    /// assert_eq!(*cell.borrow(), 15);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: &mut self guarantees exclusive access
        self.value.get_mut()
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: borrow() already checked that borrowing rules are followed
        // We hold a Ref guard, so no mutable borrows can exist
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> Drop for Ref<'_, T> {
    /// Automatically decrements the borrow count when the guard is dropped.
    /// This is RAII - cleanup happens automatically!
    fn drop(&mut self) {
        let count = self.refcell.borrow_count.get();
        self.refcell.borrow_count.set(count - 1);
    }
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: borrow_mut() already checked that borrowing rules are followed
        // We hold a RefMut guard, so no other borrows can exist
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: borrow_mut() already checked that borrowing rules are followed
        // We hold a RefMut guard, so no other borrows can exist
        unsafe { &mut *self.refcell.value.get() }
    }
}

impl<T> Drop for RefMut<'_, T> {
    /// Automatically resets the borrow count to 0 when the guard is dropped.
    fn drop(&mut self) {
        self.refcell.borrow_count.set(0);
    }
}

/// Cloning a RefCell creates an independent copy.
///
/// ```
/// use rustlib::refcell::RefCell0;
/// let cell1 = RefCell0::new(42);
/// let cell2 = cell1.clone();
///
/// *cell1.borrow_mut() = 100;
/// assert_eq!(*cell1.borrow(), 100);
/// assert_eq!(*cell2.borrow(), 42); // Independent
/// ```
impl<T: Clone> Clone for RefCell0<T> {
    fn clone(&self) -> RefCell0<T> {
        RefCell0::new(self.borrow().clone())
    }
}

/// Creates a RefCell with the default value.
///
/// ```
/// use rustlib::refcell::RefCell0;
/// let cell: RefCell0<i32> = RefCell0::default();
/// assert_eq!(*cell.borrow(), 0);
/// ```
impl<T: Default> Default for RefCell0<T> {
    fn default() -> RefCell0<T> {
        RefCell0::new(T::default())
    }
}

/// Debug formatting tries to borrow and show the value.
/// If currently borrowed, shows `<borrowed>` instead.
///
/// ```
/// use rustlib::refcell::RefCell0;
/// let cell = RefCell0::new(42);
/// assert_eq!(format!("{:?}", cell), "RefCell0(42)");
///
/// let _guard = cell.borrow_mut();
/// assert_eq!(format!("{:?}", cell), "RefCell0(<borrowed>)");
/// ```
impl<T: std::fmt::Debug> std::fmt::Debug for RefCell0<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.try_borrow() {
            Ok(borrowed) => write!(f, "RefCell0({:?})", &*borrowed),
            Err(_) => write!(f, "RefCell0(<borrowed>)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_borrow() {
        let cell = RefCell0::new(42);
        let borrowed = cell.borrow();
        assert_eq!(*borrowed, 42);
    }

    #[test]
    fn test_borrow_mut() {
        let cell = RefCell0::new(42);
        let mut borrowed = cell.borrow_mut();
        *borrowed = 100;
        drop(borrowed);

        assert_eq!(*cell.borrow(), 100);
    }

    #[test]
    fn test_multiple_immutable_borrows() {
        let cell = RefCell0::new(42);
        let r1 = cell.borrow();
        let r2 = cell.borrow();
        let r3 = cell.borrow();

        assert_eq!(*r1, 42);
        assert_eq!(*r2, 42);
        assert_eq!(*r3, 42);
    }

    #[test]
    #[should_panic(expected = "Already borrowed")]
    fn test_borrow_and_borrow_mut_panics() {
        let cell = RefCell0::new(42);
        let _r = cell.borrow();
        let _m = cell.borrow_mut(); // Should panic
    }

    #[test]
    #[should_panic(expected = "Already mutably borrowed")]
    fn test_borrow_mut_and_borrow_panics() {
        let cell = RefCell0::new(42);
        let _m = cell.borrow_mut();
        let _r = cell.borrow(); // Should panic
    }

    #[test]
    fn test_try_borrow() {
        let cell = RefCell0::new(42);
        let _m = cell.borrow_mut();

        assert!(cell.try_borrow().is_err());
    }

    #[test]
    fn test_try_borrow_mut() {
        let cell = RefCell0::new(42);
        let _r = cell.borrow();

        assert!(cell.try_borrow_mut().is_err());
    }

    #[test]
    fn test_replace() {
        let cell = RefCell0::new(42);
        let old = cell.replace(100);

        assert_eq!(old, 42);
        assert_eq!(*cell.borrow(), 100);
    }

    #[test]
    fn test_swap() {
        let cell1 = RefCell0::new(10);
        let cell2 = RefCell0::new(20);

        cell1.swap(&cell2);

        assert_eq!(*cell1.borrow(), 20);
        assert_eq!(*cell2.borrow(), 10);
    }

    #[test]
    fn test_into_inner() {
        let cell = RefCell0::new(42);
        assert_eq!(cell.into_inner(), 42);
    }

    #[test]
    fn test_clone() {
        let cell = RefCell0::new(42);
        let cell2 = cell.clone();

        assert_eq!(*cell.borrow(), *cell2.borrow());

        *cell.borrow_mut() = 100;
        assert_eq!(*cell.borrow(), 100);
        assert_eq!(*cell2.borrow(), 42); // Independent
    }

    #[test]
    fn test_default() {
        let cell: RefCell0<i32> = RefCell0::default();
        assert_eq!(*cell.borrow(), 0);
    }

    #[test]
    fn test_debug() {
        let cell = RefCell0::new(42);
        assert_eq!(format!("{:?}", cell), "RefCell0(42)");

        let _borrowed = cell.borrow_mut();
        assert_eq!(format!("{:?}", cell), "RefCell0(<borrowed>)");
    }

    #[test]
    fn test_borrow_guard_drop() {
        let cell = RefCell0::new(42);

        {
            let _r1 = cell.borrow();
            let _r2 = cell.borrow();
            // Guards dropped here
        }

        // Should be able to mutably borrow now
        let mut m = cell.borrow_mut();
        *m = 100;
    }

    #[test]
    fn test_get_mut() {
        let mut cell = RefCell0::new(5);
        *cell.get_mut() += 10;
        assert_eq!(*cell.borrow(), 15);
    }
}