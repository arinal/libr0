//! Rc0 - Educational reimplementation of `Rc<T>` and `Weak<T>`

use std::cell::Cell;
use std::mem::ManuallyDrop;
use std::ops::Deref;

/// The heap-allocated data shared by all `Rc0` and `Weak0` pointers.
struct RcInner<T: ?Sized> {
    strong_count: Cell<usize>,
    weak_count: Cell<usize>,
    value: ManuallyDrop<T>,
}

/// A reference-counted pointer for shared ownership.
pub struct Rc0<T: ?Sized> {
    ptr: *mut RcInner<T>,
}

impl<T> Rc0<T> {
    /// Creates a new reference-counted pointer.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc = Rc0::new(42);
    /// assert_eq!(*rc, 42);
    /// ```
    pub fn new(value: T) -> Rc0<T> {
        let inner = Box::new(RcInner {
            strong_count: Cell::new(1),
            weak_count: Cell::new(1),
            value: ManuallyDrop::new(value),
        });

        Rc0 {
            ptr: Box::into_raw(inner),
        }
    }

    /// Returns the number of strong references.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc = Rc0::new(42);
    /// assert_eq!(Rc0::strong_count(&rc), 1);
    /// let rc2 = Rc0::clone(&rc);
    /// assert_eq!(Rc0::strong_count(&rc), 2);
    /// ```
    pub fn strong_count(this: &Rc0<T>) -> usize {
        unsafe { (*this.ptr).strong_count.get() }
    }

    /// Returns the number of weak references.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc = Rc0::new(42);
    /// assert_eq!(Rc0::weak_count(&rc), 0);
    /// let weak = Rc0::downgrade(&rc);
    /// assert_eq!(Rc0::weak_count(&rc), 1);
    /// ```
    pub fn weak_count(this: &Rc0<T>) -> usize {
        let count = unsafe { (*this.ptr).weak_count.get() };
        if count > 0 {
            count - 1
        } else {
            0
        }
    }

    /// Creates a new weak reference.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc = Rc0::new(String::from("hello"));
    /// let weak = Rc0::downgrade(&rc);
    /// assert!(weak.upgrade().is_some());
    /// ```
    pub fn downgrade(this: &Rc0<T>) -> Weak0<T> {
        let inner = unsafe { &*this.ptr };
        inner.weak_count.set(inner.weak_count.get() + 1);
        Weak0 { ptr: this.ptr }
    }

    /// Returns a mutable reference if this is the only strong reference.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let mut rc = Rc0::new(5);
    /// *Rc0::get_mut(&mut rc).unwrap() = 10;
    /// assert_eq!(*rc, 10);
    /// ```
    pub fn get_mut(this: &mut Rc0<T>) -> Option<&mut T> {
        if Rc0::strong_count(this) == 1 {
            // SAFETY: We're the sole owner
            unsafe { Some(&mut (*this.ptr).value) }
        } else {
            None
        }
    }

    /// Returns true if the two pointers point to the same allocation.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc1 = Rc0::new(42);
    /// let rc2 = Rc0::clone(&rc1);
    /// assert!(Rc0::ptr_eq(&rc1, &rc2));
    /// ```
    pub fn ptr_eq(a: &Rc0<T>, b: &Rc0<T>) -> bool {
        a.ptr == b.ptr
    }
}

impl<T: ?Sized> Clone for Rc0<T> {
    /// Clones the reference-counted pointer (increments the count).
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc1 = Rc0::new(42);
    /// let rc2 = rc1.clone();
    /// assert_eq!(Rc0::strong_count(&rc1), 2);
    /// ```
    fn clone(&self) -> Rc0<T> {
        let inner = unsafe { &*self.ptr };
        inner.strong_count.set(inner.strong_count.get() + 1);
        Rc0 { ptr: self.ptr }
    }
}

impl<T: ?Sized> Deref for Rc0<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: Rc always points to valid data
        unsafe { &(*self.ptr).value }
    }
}

impl<T: ?Sized> Drop for Rc0<T> {
    fn drop(&mut self) {
        let inner = unsafe { &*self.ptr };
        let strong = inner.strong_count.get();
        inner.strong_count.set(strong - 1);

        if strong == 1 {
            // Drop the value
            unsafe {
                std::ptr::drop_in_place(&mut (*self.ptr).value as *mut ManuallyDrop<T> as *mut T);
            }

            // Decrement weak count (remove implicit weak ref)
            let weak = inner.weak_count.get();
            inner.weak_count.set(weak - 1);

            if weak == 1 {
                // Deallocate
                unsafe {
                    drop(Box::from_raw(self.ptr));
                }
            }
        }
    }
}

impl<T: Default> Default for Rc0<T> {
    /// Creates a new Rc with the default value.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc: Rc0<i32> = Rc0::default();
    /// assert_eq!(*rc, 0);
    /// ```
    fn default() -> Rc0<T> {
        Rc0::new(T::default())
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Rc0<T> {
    /// Formats the value using the given formatter.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc = Rc0::new(42);
    /// assert_eq!(format!("{:?}", rc), "42");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

/// A weak reference that doesn't own the value.
pub struct Weak0<T: ?Sized> {
    ptr: *mut RcInner<T>,
}

impl<T> Weak0<T> {
    /// Creates a new empty weak reference that doesn't point to anything.
    /// ```
    /// use rustlib::rc::Weak0;
    /// let weak: Weak0<i32> = Weak0::new();
    /// assert!(weak.upgrade().is_none());
    /// ```
    pub fn new() -> Weak0<T> {
        Weak0 {
            ptr: std::ptr::null_mut(),
        }
    }
}

impl<T: ?Sized> Weak0<T> {
    /// Attempts to upgrade to a strong reference.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc = Rc0::new(42);
    /// let weak = Rc0::downgrade(&rc);
    /// assert!(weak.upgrade().is_some());
    /// drop(rc);
    /// assert!(weak.upgrade().is_none());
    /// ```
    pub fn upgrade(&self) -> Option<Rc0<T>> {
        if self.ptr.is_null() {
            return None;
        }

        let inner = unsafe { &*self.ptr };
        let strong = inner.strong_count.get();

        if strong == 0 {
            None
        } else {
            inner.strong_count.set(strong + 1);
            Some(Rc0 { ptr: self.ptr })
        }
    }

    /// Returns the number of strong references.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc = Rc0::new(42);
    /// let weak = Rc0::downgrade(&rc);
    /// assert_eq!(weak.strong_count(), 1);
    /// ```
    pub fn strong_count(&self) -> usize {
        if self.ptr.is_null() {
            0
        } else {
            unsafe { (*self.ptr).strong_count.get() }
        }
    }

    /// Returns the number of weak references.
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc = Rc0::new(42);
    /// let weak = Rc0::downgrade(&rc);
    /// assert_eq!(weak.weak_count(), 1);
    /// ```
    pub fn weak_count(&self) -> usize {
        if self.ptr.is_null() {
            0
        } else {
            let count = unsafe { (*self.ptr).weak_count.get() };
            if count > 0 {
                count - 1
            } else {
                0
            }
        }
    }

    /// Returns true if the two pointers point to the same allocation.
    /// ```
    /// use rustlib::rc::{Rc0, Weak0};
    /// let rc = Rc0::new(42);
    /// let weak1 = Rc0::downgrade(&rc);
    /// let weak2 = weak1.clone();
    /// assert!(Weak0::ptr_eq(&weak1, &weak2));
    /// ```
    pub fn ptr_eq(a: &Weak0<T>, b: &Weak0<T>) -> bool {
        a.ptr == b.ptr
    }
}

impl<T: ?Sized> Clone for Weak0<T> {
    /// Clones the weak reference (increments the weak count).
    /// ```
    /// use rustlib::rc::Rc0;
    /// let rc = Rc0::new(42);
    /// let weak1 = Rc0::downgrade(&rc);
    /// let weak2 = weak1.clone();
    /// assert_eq!(Rc0::weak_count(&rc), 2);
    /// ```
    fn clone(&self) -> Weak0<T> {
        if !self.ptr.is_null() {
            let inner = unsafe { &*self.ptr };
            inner.weak_count.set(inner.weak_count.get() + 1);
        }
        Weak0 { ptr: self.ptr }
    }
}

impl<T: ?Sized> Drop for Weak0<T> {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }

        let inner = unsafe { &*self.ptr };
        let weak = inner.weak_count.get();
        inner.weak_count.set(weak - 1);

        if weak == 1 && inner.strong_count.get() == 0 {
            // Deallocate
            unsafe {
                drop(Box::from_raw(self.ptr));
            }
        }
    }
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for Weak0<T> {
    /// Formats the value using the given formatter.
    /// ```
    /// use rustlib::rc::{Rc0, Weak0};
    /// let rc = Rc0::new(42);
    /// let weak = Rc0::downgrade(&rc);
    /// assert!(format!("{:?}", weak).contains("(Weak)"));
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(Weak)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let rc = Rc0::new(42);
        assert_eq!(*rc, 42);
        assert_eq!(Rc0::strong_count(&rc), 1);
    }

    #[test]
    fn test_clone() {
        let rc1 = Rc0::new(String::from("hello"));
        let rc2 = Rc0::clone(&rc1);
        assert_eq!(*rc1, "hello");
        assert_eq!(*rc2, "hello");
        assert_eq!(Rc0::strong_count(&rc1), 2);
    }

    #[test]
    fn test_drop() {
        let rc1 = Rc0::new(42);
        let rc2 = Rc0::clone(&rc1);
        assert_eq!(Rc0::strong_count(&rc1), 2);
        drop(rc2);
        assert_eq!(Rc0::strong_count(&rc1), 1);
    }

    #[test]
    fn test_weak() {
        let rc = Rc0::new(42);
        let weak = Rc0::downgrade(&rc);
        assert_eq!(Rc0::weak_count(&rc), 1);
        assert_eq!(*weak.upgrade().unwrap(), 42);
        drop(rc);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_get_mut() {
        let mut rc = Rc0::new(5);
        *Rc0::get_mut(&mut rc).unwrap() = 10;
        assert_eq!(*rc, 10);
    }

    #[test]
    fn test_get_mut_shared() {
        let mut rc1 = Rc0::new(5);
        let _rc2 = Rc0::clone(&rc1);
        assert!(Rc0::get_mut(&mut rc1).is_none());
    }

    #[test]
    fn test_ptr_eq() {
        let rc1 = Rc0::new(42);
        let rc2 = Rc0::clone(&rc1);
        let rc3 = Rc0::new(42);
        assert!(Rc0::ptr_eq(&rc1, &rc2));
        assert!(!Rc0::ptr_eq(&rc1, &rc3));
    }
}

