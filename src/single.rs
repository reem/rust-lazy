use std::ops::{Deref, DerefMut};
use std::cell::UnsafeCell;
use std::ptr;
use std::thunk::Invoke;

use self::Inner::{Evaluated, EvaluationInProgress, Unevaluated};

/// A sometimes-cleaner name for a lazily evaluated value.
pub type Lazy<T> = Thunk<T>;

/// A lazily evaluated value.
pub struct Thunk<T> {
    inner: UnsafeCell<Inner<T>>,
}

impl<T> !Sync for Thunk<T> {}

impl<T> Thunk<T> {
    /// Create a lazily evaluated value from a proc that returns that value.
    ///
    /// You can construct Thunk's manually using this, but the lazy! macro
    /// is preferred.
    ///
    /// ```rust
    /// # use lazy::single::Thunk;
    /// let expensive = Thunk::new(|| { println!("Evaluated!"); 7u });
    /// assert_eq!(*expensive, 7u); // "Evaluated!" gets printed here.
    /// assert_eq!(*expensive, 7u); // Nothing printed.
    /// ```
    pub fn new<F>(producer: F) -> Thunk<T>
    where F: 'static + FnOnce() -> T {
        Thunk {
            inner: UnsafeCell::new(Unevaluated(Producer::new(producer))),
        }
    }


    /// Create a new, evaluated, thunk from a value.
    pub fn evaluated(val: T) -> Thunk<T> {
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
                Evaluated(val) => { val },
                _ => debug_unreachable!()
            }
        }
    }
}

struct Producer<T> {
    inner: Box<Invoke<(), T> + 'static>
}

impl<T> Producer<T> {
    fn new<F: 'static + FnOnce() -> T>(f: F) -> Producer<T> {
        Producer {
            inner: Box::new(move |()| {
                f()
            }) as Box<Invoke<(), T> + 'static>
        }
    }

    fn invoke(self) -> T {
        self.inner.invoke(())
    }
}

enum Inner<T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(Producer<T>)
}

impl<T> Deref for Thunk<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        self.force();
        match unsafe { &*self.inner.get() } {
            &Evaluated(ref val) => val,
            _ => unsafe { debug_unreachable!() }
        }
    }
}

impl<T> DerefMut for Thunk<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        self.force();
        match unsafe { &mut *self.inner.get() } {
            &mut Evaluated(ref mut val) => val,
            _ => unsafe { debug_unreachable!() }
        }
    }
}

