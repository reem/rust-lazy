#![license = "MIT"]
#![deny(missing_docs)]
#![deny(warnings)]

#![feature(unsafe_destructor, macro_rules)]

//! Lazy evaluation for Rust.

pub use self::single::{Lazy, Thunk};
pub use self::sync::{SyncLazy, SyncThunk};

mod single;
mod sync;

#[macro_export]
macro_rules! lazy (
    ($e:expr) => {
        Thunk::new(proc() { $e })
    }
)

#[macro_export]
macro_rules! sync_lazy (
    ($e:expr) => {
        SyncThunk::new(proc() { $e })
    }
)

