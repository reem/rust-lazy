 #![license = "MIT"]
 #![deny(missing_doc)]
 #![deny(warnings)]

#![feature(unsafe_destructor, macro_rules)]

//! Lazy evaluation for Rust.

use std::mem::{transmute, uninitialized, forget};
use std::ptr::{replace, copy_nonoverlapping_memory};

/// A lazily evaluated value.
pub struct Thunk<T> {
    inner: *mut Inner<T>,
}

#[macro_exports]
macro_rules! lazy (
    ($e:expr) => {
        Thunk::new(proc() { $e })
    }
)

impl<T> Thunk<T> {
    /// Create a lazily evaluated value from a proc that returns that value.
    pub fn new(producer: proc() -> T) -> Thunk<T> {
        Thunk { inner: unsafe { transmute(box Unevaluated(producer)) } }
    }

    /// Force evaluation of a thunk.
    pub fn force(&self) {
        let mut inner = unsafe { uninitialized() };
        unsafe { copy_nonoverlapping_memory(&mut inner, self.inner as *const Inner<T>, 1) };
        match inner {
            Evaluated(val) => unsafe { forget(val) },
            Unevaluated(producer) => unsafe {
                forget(replace(self.inner, Evaluated(producer())));
            }
        }
    }
}

enum Inner<T> {
    Evaluated(T),
    Unevaluated(proc() -> T)
}

impl<T> Deref<T> for Thunk<T> {
    fn deref<'a>(&'a self) -> &'a T {
        self.force();
        match unsafe { &*self.inner } {
            &Evaluated(ref val) => val,
            _ => unreachable!()
        }
    }
}

impl<T> DerefMut<T> for Thunk<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        self.force();
        match unsafe { &mut *self.inner } {
            &Evaluated(ref mut val) => val,
            _ => unreachable!()
        }
    }
}

#[unsafe_destructor]
impl<T> Drop for Thunk<T> {
    fn drop(&mut self) {
        let inner: Box<Inner<T>> = unsafe { transmute(self.inner) };
        drop(inner);
    }
}

#[cfg(test)]
mod test;

