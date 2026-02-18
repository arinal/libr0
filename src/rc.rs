//! Rc0 - Educational reimplementation of Rc<T>

use std::cell::Cell;
use std::mem::ManuallyDrop;
use std::ops::Deref;

struct RcInner<T> {
    strong_count: Cell<usize>,
    weak_count: Cell<usize>,
    // ManuallyDrop prevents double-free when we deallocate via Box::from_raw
    // We manually drop the value when strong_count reaches 0
    value: ManuallyDrop<T>,
}

pub struct Rc0<T> {
    ptr: *mut RcInner<T>,
}

pub struct Weak0<T> {
    ptr: *mut RcInner<T>,
}

impl<T> Rc0<T> {
    pub fn new(value: T) -> Rc0<T> {
        let inner = Box::new(RcInner {
            strong_count: Cell::new(1),
            weak_count: Cell::new(1), // Implicit weak ref for strong refs
            value: ManuallyDrop::new(value),
        });
        Rc0 {
            ptr: Box::into_raw(inner),
        }
    }

    pub fn strong_count(this: &Rc0<T>) -> usize {
        unsafe { (*this.ptr).strong_count.get() }
    }

    pub fn weak_count(this: &Rc0<T>) -> usize {
        // Subtract the implicit weak ref
        unsafe { (*this.ptr).weak_count.get() - 1 }
    }

    pub fn downgrade(this: &Rc0<T>) -> Weak0<T> {
        let inner = unsafe { &*this.ptr };
        inner.weak_count.set(inner.weak_count.get() + 1);
        Weak0 { ptr: this.ptr }
    }

    pub fn get_mut(this: &mut Rc0<T>) -> Option<&mut T> {
        if Rc0::strong_count(this) == 1 && Rc0::weak_count(this) == 0 {
            unsafe { Some(&mut (*this.ptr).value) }
        } else {
            None
        }
    }

    pub fn ptr_eq(a: &Rc0<T>, b: &Rc0<T>) -> bool {
        a.ptr == b.ptr
    }
}

impl<T> Clone for Rc0<T> {
    fn clone(&self) -> Rc0<T> {
        let inner = unsafe { &*self.ptr };
        inner.strong_count.set(inner.strong_count.get() + 1);
        Rc0 { ptr: self.ptr }
    }
}

impl<T> Deref for Rc0<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &(*self.ptr).value }
    }
}

impl<T> Drop for Rc0<T> {
    fn drop(&mut self) {
        let inner = unsafe { &*self.ptr };
        let count = inner.strong_count.get();

        if count == 1 {
            // Last strong reference - drop the value first
            unsafe { ManuallyDrop::drop(&mut (*self.ptr).value) };
        }

        inner.strong_count.set(count - 1);

        if count == 1 {
            // Decrement the implicit weak ref
            let weak = inner.weak_count.get();
            inner.weak_count.set(weak - 1);

            // If no weak refs remain, deallocate
            if weak == 1 {
                drop(unsafe { Box::from_raw(self.ptr) });
            }
        }
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Rc0<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rc0({:?})", **self)
    }
}

// ============================================================================
// Weak implementation
// ============================================================================

impl<T> Weak0<T> {
    pub fn upgrade(&self) -> Option<Rc0<T>> {
        let inner = unsafe { &*self.ptr };
        if inner.strong_count.get() == 0 {
            None
        } else {
            inner.strong_count.set(inner.strong_count.get() + 1);
            Some(Rc0 { ptr: self.ptr })
        }
    }

    pub fn strong_count(&self) -> usize {
        unsafe { (*self.ptr).strong_count.get() }
    }
}

impl<T> Clone for Weak0<T> {
    fn clone(&self) -> Weak0<T> {
        let inner = unsafe { &*self.ptr };
        inner.weak_count.set(inner.weak_count.get() + 1);
        Weak0 { ptr: self.ptr }
    }
}

impl<T> Drop for Weak0<T> {
    fn drop(&mut self) {
        let inner = unsafe { &*self.ptr };
        let weak = inner.weak_count.get();
        inner.weak_count.set(weak - 1);

        // Deallocate if both counts are zero
        if weak == 1 && inner.strong_count.get() == 0 {
            drop(unsafe { Box::from_raw(self.ptr) });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_deref() {
        let rc = Rc0::new(42);
        assert_eq!(*rc, 42);
    }

    #[test]
    fn test_clone() {
        let rc1 = Rc0::new(42);
        let rc2 = rc1.clone();

        assert_eq!(*rc1, 42);
        assert_eq!(*rc2, 42);
        assert_eq!(Rc0::strong_count(&rc1), 2);
    }

    #[test]
    fn test_strong_count() {
        let rc1 = Rc0::new(42);
        assert_eq!(Rc0::strong_count(&rc1), 1);

        let rc2 = rc1.clone();
        assert_eq!(Rc0::strong_count(&rc1), 2);
        assert_eq!(Rc0::strong_count(&rc2), 2);

        drop(rc2);
        assert_eq!(Rc0::strong_count(&rc1), 1);
    }

    #[test]
    fn test_ptr_eq() {
        let rc1 = Rc0::new(42);
        let rc2 = rc1.clone();
        let rc3 = Rc0::new(42);

        assert!(Rc0::ptr_eq(&rc1, &rc2));
        assert!(!Rc0::ptr_eq(&rc1, &rc3));
    }

    #[test]
    fn test_get_mut() {
        let mut rc1 = Rc0::new(42);

        // Single owner, should get mutable reference
        if let Some(val) = Rc0::get_mut(&mut rc1) {
            *val = 100;
        }
        assert_eq!(*rc1, 100);

        // Multiple owners, should return None
        let _rc2 = rc1.clone();
        assert!(Rc0::get_mut(&mut rc1).is_none());
    }

    #[test]
    fn test_downgrade() {
        let rc = Rc0::new(42);
        let weak = Rc0::downgrade(&rc);

        assert_eq!(Rc0::weak_count(&rc), 1);
        assert_eq!(weak.strong_count(), 1);
    }

    #[test]
    fn test_weak_upgrade() {
        let rc = Rc0::new(42);
        let weak = Rc0::downgrade(&rc);

        let upgraded = weak.upgrade();
        assert!(upgraded.is_some());
        assert_eq!(*upgraded.unwrap(), 42);
    }

    #[test]
    fn test_weak_upgrade_after_drop() {
        let rc = Rc0::new(42);
        let weak = Rc0::downgrade(&rc);

        drop(rc);

        let upgraded = weak.upgrade();
        assert!(upgraded.is_none());
    }

    #[test]
    fn test_weak_clone() {
        let rc = Rc0::new(42);
        let weak1 = Rc0::downgrade(&rc);
        let _weak2 = weak1.clone();

        assert_eq!(Rc0::weak_count(&rc), 2);
    }

    #[test]
    fn test_drop_with_weak_refs() {
        let rc = Rc0::new(String::from("hello"));
        let weak = Rc0::downgrade(&rc);

        drop(rc);

        // Weak ref should still exist but upgrade should fail
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_debug() {
        let rc = Rc0::new(42);
        assert_eq!(format!("{:?}", rc), "Rc0(42)");
    }

    #[test]
    fn test_multiple_weak_refs() {
        let rc = Rc0::new(42);
        let weak1 = Rc0::downgrade(&rc);
        let weak2 = Rc0::downgrade(&rc);
        let weak3 = weak1.clone();

        assert_eq!(Rc0::weak_count(&rc), 3);

        drop(weak1);
        assert_eq!(Rc0::weak_count(&rc), 2);

        drop(weak2);
        assert_eq!(Rc0::weak_count(&rc), 1);

        drop(weak3);
        assert_eq!(Rc0::weak_count(&rc), 0);
    }

    #[test]
    fn test_drop_order() {
        use std::sync::Arc;
        let drop_checker = Arc::new(());
        assert_eq!(Arc::strong_count(&drop_checker), 1);

        {
            let rc1 = Rc0::new(drop_checker.clone());
            let rc2 = rc1.clone();
            let rc3 = rc1.clone();

            assert_eq!(Arc::strong_count(&drop_checker), 2); // 1 original + 1 in Rc

            drop(rc1);
            assert_eq!(Arc::strong_count(&drop_checker), 2); // Still in Rc

            drop(rc2);
            assert_eq!(Arc::strong_count(&drop_checker), 2); // Still in Rc

            drop(rc3);
            assert_eq!(Arc::strong_count(&drop_checker), 1); // Rc dropped, back to original
        }
    }
}