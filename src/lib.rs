 #![license = "MIT"]
 #![deny(missing_doc)]
 #![deny(warnings)]

#![feature(unsafe_destructor, macro_rules)]

//! Lazy evaluation for Rust.

use std::mem::{uninitialized, forget};
use std::ptr::{replace, copy_nonoverlapping_memory};
use std::cell::UnsafeCell;

/// A sometimes-cleaner name for a lazily evaluated value.
pub type Lazy<T> = Thunk<T>;

/// A lazily evaluated value.
pub struct Thunk<T> {
    inner: UnsafeCell<Inner<T>>,
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
        Thunk { inner: UnsafeCell::new(Unevaluated(producer)) }
    }

    /// Force evaluation of a thunk.
    pub fn force(&self) {
        unsafe {
            let mut inner = uninitialized();
            copy_nonoverlapping_memory(&mut inner, self.inner.get() as *const Inner<T>, 1);
            match inner {
                Evaluated(val) => { forget(val) },
                Unevaluated(producer) => {
                    forget(replace(self.inner.get(), Evaluated(producer())));
                }
            }
        }
    }

    /// Force the evaluation of a thunk and get the value, consuming the thunk.
    pub fn unwrap(self) -> T {
        self.force();
        unsafe {
            match self.inner.unwrap() {
                Evaluated(val) => { val },
                _ => unreachable!()
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
        match unsafe { &*self.inner.get() } {
            &Evaluated(ref val) => val,
            _ => unreachable!()
        }
    }
}

impl<T> DerefMut<T> for Thunk<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        self.force();
        match unsafe { &mut *self.inner.get() } {
            &Evaluated(ref mut val) => val,
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
mod test;

