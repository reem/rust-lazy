 #![license = "MIT"]
 #![deny(missing_doc)]
 #![deny(warnings)]

#![feature(unsafe_destructor)]

//! Lazy evaluation for Rust.

use std::mem::{transmute, uninitialized};
use std::ptr::replace;

/// A lazily evaluated value.
pub struct Thunk<T> {
    inner: *mut Inner<T>,
}

enum Inner<T> {
    Evaluated(T),
    Unevaluated(proc() -> T)
}

impl<T> Inner<T> {
    fn unwrap<'a>(&'a self) -> &'a T {
        match *self {
            Evaluated(ref val) => val,
            Unevaluated(_) => fail!("Unwrapped an unevaluated inner thunk.")
        }
    }

    fn unwrap_mut<'a>(&'a mut self) -> &'a mut T {
        match *self {
            Evaluated(ref mut val) => val,
            Unevaluated(_) => fail!("Unwrapped an unevaluated inner thunk.")
        }
    }
}

impl<T> Thunk<T> {
    /// Create a lazily evaluated value from a proc that returns that value.
    pub fn new(producer: proc() -> T) -> Thunk<T> {
        Thunk { inner: unsafe { transmute(box Unevaluated(producer)) } }
    }
}

impl<T> Deref<T> for Thunk<T> {
    fn deref<'a>(&'a self) -> &'a T {
        let inner = unsafe { replace(self.inner, uninitialized()) };
        match inner {
            Evaluated(val) => unsafe {
                *self.inner = Evaluated(val);
            },
            Unevaluated(producer) => unsafe {
                *self.inner = Evaluated(producer());
            }
        }
        unsafe { (*self.inner).unwrap() }
    }
}

impl<T> DerefMut<T> for Thunk<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        let inner = unsafe { replace(self.inner, uninitialized()) };
        match inner {
            Evaluated(val) => unsafe {
                *self.inner = Evaluated(val);
            },
            Unevaluated(producer) => unsafe {
                *self.inner = Evaluated(producer());
            }
        }
        unsafe { (*self.inner).unwrap_mut() }
    }
}

#[unsafe_destructor]
impl<T> Drop for Thunk<T> {
    fn drop(&mut self) {
        let inner: Box<Inner<T>> = unsafe { transmute(self.inner) };
        drop(inner);
    }
}

