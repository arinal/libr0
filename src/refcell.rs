//! MyRefCell - Educational reimplementation of RefCell<T>

use std::cell::{Cell, UnsafeCell};
use std::ops::{Deref, DerefMut};

pub struct MyRefCell<T> {
    borrow_count: Cell<isize>,
    value: UnsafeCell<T>,
}

pub struct Ref<'a, T> {
    refcell: &'a MyRefCell<T>,
}

pub struct RefMut<'a, T> {
    refcell: &'a MyRefCell<T>,
}

#[derive(Debug)]
pub struct BorrowError;

#[derive(Debug)]
pub struct BorrowMutError;

impl<T> MyRefCell<T> {
    pub fn new(value: T) -> MyRefCell<T> {
        MyRefCell {
            borrow_count: Cell::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        self.try_borrow().expect("Already mutably borrowed")
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.try_borrow_mut().expect("Already borrowed")
    }

    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        let count = self.borrow_count.get();
        if count < 0 {
            Err(BorrowError)
        } else {
            self.borrow_count.set(count + 1);
            Ok(Ref { refcell: self })
        }
    }

    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        if self.borrow_count.get() != 0 {
            Err(BorrowMutError)
        } else {
            self.borrow_count.set(-1);
            Ok(RefMut { refcell: self })
        }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }

    pub fn replace(&self, value: T) -> T {
        std::mem::replace(&mut *self.borrow_mut(), value)
    }

    pub fn swap(&self, other: &MyRefCell<T>) {
        std::mem::swap(&mut *self.borrow_mut(), &mut *other.borrow_mut())
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        let count = self.refcell.borrow_count.get();
        self.refcell.borrow_count.set(count - 1);
    }
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.refcell.value.get() }
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.refcell.value.get() }
    }
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        self.refcell.borrow_count.set(0);
    }
}

impl<T: Clone> Clone for MyRefCell<T> {
    fn clone(&self) -> MyRefCell<T> {
        MyRefCell::new(self.borrow().clone())
    }
}

impl<T: Default> Default for MyRefCell<T> {
    fn default() -> MyRefCell<T> {
        MyRefCell::new(T::default())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for MyRefCell<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.try_borrow() {
            Ok(borrowed) => write!(f, "MyRefCell({:?})", &*borrowed),
            Err(_) => write!(f, "MyRefCell(<borrowed>)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_borrow() {
        let cell = MyRefCell::new(42);
        let borrowed = cell.borrow();
        assert_eq!(*borrowed, 42);
    }

    #[test]
    fn test_borrow_mut() {
        let cell = MyRefCell::new(42);
        let mut borrowed = cell.borrow_mut();
        *borrowed = 100;
        drop(borrowed);

        assert_eq!(*cell.borrow(), 100);
    }

    #[test]
    fn test_multiple_immutable_borrows() {
        let cell = MyRefCell::new(42);
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
        let cell = MyRefCell::new(42);
        let _r = cell.borrow();
        let _m = cell.borrow_mut(); // Should panic
    }

    #[test]
    #[should_panic(expected = "Already mutably borrowed")]
    fn test_borrow_mut_and_borrow_panics() {
        let cell = MyRefCell::new(42);
        let _m = cell.borrow_mut();
        let _r = cell.borrow(); // Should panic
    }

    #[test]
    fn test_try_borrow() {
        let cell = MyRefCell::new(42);
        let _m = cell.borrow_mut();

        assert!(cell.try_borrow().is_err());
    }

    #[test]
    fn test_try_borrow_mut() {
        let cell = MyRefCell::new(42);
        let _r = cell.borrow();

        assert!(cell.try_borrow_mut().is_err());
    }

    #[test]
    fn test_replace() {
        let cell = MyRefCell::new(42);
        let old = cell.replace(100);

        assert_eq!(old, 42);
        assert_eq!(*cell.borrow(), 100);
    }

    #[test]
    fn test_swap() {
        let cell1 = MyRefCell::new(10);
        let cell2 = MyRefCell::new(20);

        cell1.swap(&cell2);

        assert_eq!(*cell1.borrow(), 20);
        assert_eq!(*cell2.borrow(), 10);
    }

    #[test]
    fn test_into_inner() {
        let cell = MyRefCell::new(42);
        assert_eq!(cell.into_inner(), 42);
    }

    #[test]
    fn test_clone() {
        let cell = MyRefCell::new(42);
        let cell2 = cell.clone();

        assert_eq!(*cell.borrow(), *cell2.borrow());

        *cell.borrow_mut() = 100;
        assert_eq!(*cell.borrow(), 100);
        assert_eq!(*cell2.borrow(), 42); // Independent
    }

    #[test]
    fn test_default() {
        let cell: MyRefCell<i32> = MyRefCell::default();
        assert_eq!(*cell.borrow(), 0);
    }

    #[test]
    fn test_debug() {
        let cell = MyRefCell::new(42);
        assert_eq!(format!("{:?}", cell), "MyRefCell(42)");

        let _borrowed = cell.borrow_mut();
        assert_eq!(format!("{:?}", cell), "MyRefCell(<borrowed>)");
    }

    #[test]
    fn test_borrow_guard_drop() {
        let cell = MyRefCell::new(42);

        {
            let _r1 = cell.borrow();
            let _r2 = cell.borrow();
            // Guards dropped here
        }

        // Should be able to mutably borrow now
        let mut m = cell.borrow_mut();
        *m = 100;
    }
}