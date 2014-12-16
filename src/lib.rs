#![deny(missing_docs, warnings)]
#![feature(unsafe_destructor, macro_rules, phase, globs, default_type_params)]

//! Lazy evaluation for Rust.

#[phase(plugin)] extern crate debug_unreachable;
extern crate oncemutex;

/// A Thunk safe for single-threaded access.
pub mod single;

/// A Thunk safe for multi-threaded use.
pub mod sync;

mod lazy {
    pub use super::*;
}

#[macro_export]
macro_rules! lazy (
    ($e:expr) => {
        ::lazy::single::Thunk::new(move |:| { $e })
    }
)

#[macro_export]
macro_rules! sync_lazy (
    ($e:expr) => {
        ::lazy::sync::Thunk::new(move |:| { $e })
    }
)

