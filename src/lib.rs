 #![license = "MIT"]
 #![deny(missing_doc)]
 #![deny(warnings)]

#![feature(unsafe_destructor, macro_rules)]

//! Lazy evaluation for Rust.

pub use self::single::{Lazy, Thunk};
//pub use self::shared::{SharedLazy, SharedThunk};

mod single;
//mod shared;

#[macro_export]
macro_rules! lazy (
    ($e:expr) => {
        Thunk::new(proc() { $e })
    }
)

//#[macro_export]
//macro_rules! shared_lazy (
//    ($e:expr) => {
//        SharedThunk::new(proc() { $e })
//    }
//)


#[cfg(test)]
mod test;

