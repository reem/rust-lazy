use std::ops::{Deref, DerefMut};
use std::cell::UnsafeCell;
use std::ptr;

use self::Inner::{Evaluated, EvaluationInProgress, Unevaluated};
use fnbox::FnBox;

/// A sometimes-cleaner name for a lazily evaluated value.
pub type Lazy<'a, T> = Thunk<'a, T>;

/// A lazily evaluated value.
pub struct Thunk<'a, T> {
    inner: UnsafeCell<Inner<'a, T>>,
}

impl<'a, T> Thunk<'a, T> {
    /// Create a lazily evaluated value from a proc that returns that value.
    ///
    /// You can construct Thunk's manually using this, but the lazy! macro
    /// is preferred.
    ///
    /// ```rust
    /// # use lazy::single::Thunk;
    /// let expensive = Thunk::new(|| { println!("Evaluated!"); 7 });
    /// assert_eq!(*expensive, 7); // "Evaluated!" gets printed here.
    /// assert_eq!(*expensive, 7); // Nothing printed.
    /// ```
    pub fn new<F>(producer: F) -> Thunk<'a, T>
    where F: 'a + FnOnce() -> T {
        Thunk {
            inner: UnsafeCell::new(Unevaluated(Producer::new(producer))),
        }
    }


    /// Create a new, evaluated, thunk from a value.
    pub fn evaluated<'b>(val: T) -> Thunk<'b, T> {
        Thunk { inner: UnsafeCell::new(Evaluated(val)) }
    }

    /// Force evaluation of a thunk.
    pub fn force(&self) {
        unsafe {
            match *self.inner.get() {
                Evaluated(_) => return,
                EvaluationInProgress => {
                    panic!("Thunk::force called recursively. (A Thunk tried to force itself while trying to force itself).")
                },
                Unevaluated(_) => ()
            }

            match ptr::replace(self.inner.get(), EvaluationInProgress) {
                Unevaluated(producer) => *self.inner.get() = Evaluated(producer.invoke()),
                _ => debug_unreachable!()
            };
        }
    }

    /// Force the evaluation of a thunk and get the value, consuming the thunk.
    pub fn unwrap(self) -> T {
        self.force();
        unsafe {
            match self.inner.into_inner() {
                Evaluated(val) => val,
                _ => debug_unreachable!()
            }
        }
    }
}

struct Producer<'a, T> {
    inner: Box<FnBox<Output=T> + 'a>
}

impl<'a,T> Producer<'a,T> {
    fn new<F: 'a + FnOnce() -> T>(f: F) -> Producer<'a,T> {
        Producer { inner: Box::new(f) }
    }

    fn invoke(self) -> T {
        self.inner.call_box(())
    }
}

enum Inner<'a, T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(Producer<'a, T>)
}

impl<'x, T> Deref for Thunk<'x, T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        self.force();
        match unsafe { &*self.inner.get() } {
            &Evaluated(ref val) => val,
            _ => unsafe { debug_unreachable!() }
        }
    }
}

impl<'x, T> DerefMut for Thunk<'x, T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        self.force();
        match unsafe { &mut *self.inner.get() } {
            &mut Evaluated(ref mut val) => val,
            _ => unsafe { debug_unreachable!() }
        }
    }
}
