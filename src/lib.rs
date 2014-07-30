 #![license = "MIT"]
 #![deny(missing_doc)]
 #![deny(warnings)]

//! Crate comment goes here

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


impl<T> Thunk<T> {
    /// Create a lazily evaluated value from a proc that returns that value.
    pub fn new(producer: proc() -> T) -> Thunk<T> {
        Thunk { inner: unsafe { transmute(box Unevaluated(producer)) } }
    }
}


