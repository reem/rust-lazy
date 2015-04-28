use std::ops::{Deref, DerefMut};
use oncemutex::OnceMutex;
use std::mem;

use self::Inner::{Evaluated, EvaluationInProgress, Unevaluated};
use fnbox::FnBox;

/// A sometimes cleaner name.
pub type Lazy<'a,T> = Thunk<'a,T>;

/// Sync, Send lazy data.
pub struct Thunk<'a, T> {
    inner: OnceMutex<Inner<'a, T>>
}

unsafe impl<'a, T: Sync> Sync for Thunk<'a, T> {}

impl<'a, T: Send + Sync> Thunk<'a, T> {
    /// Create a new sync thunk.
    ///
    /// You can construct Thunk's manually using this, but the
    /// sync_lazy! macro is preferred.
    ///
    /// ```rust
    /// # use lazy::sync::Thunk;
    /// # use std::sync::Arc;
    /// # use std::thread;
    /// let expensive = Thunk::new(|| { println!("Evaluated!"); 7 });
    /// let reff = Arc::new(expensive);
    /// let reff_clone = reff.clone();
    ///
    /// // Evaluated is printed sometime beneath this line.
    /// thread::spawn(move || {
    ///     assert_eq!(**reff_clone, 7);
    /// });
    /// assert_eq!(**reff, 7);
    /// ```
    pub fn new<F: 'a>(producer: F) -> Thunk<'a, T>
    where F: Send + Sync + FnOnce() -> T {
        Thunk {
            inner: OnceMutex::new(Unevaluated(Producer::new(producer)))
        }
    }

    /// Create a new, evaluated, thunk from a value.
    pub fn evaluated(val: T) -> Thunk<'a, T> {
        let mutex = OnceMutex::new(Evaluated(val));

        // Since we use the invariants of the OnceMutex later,
        // we have to ensure that they are upheld in this case
        // by using up our lock.
        mutex.lock();

        Thunk { inner: mutex }
    }

    /// Force evaluation of a thunk.
    pub fn force(&self) {
        match self.inner.lock() {
            // We are the thread responsible for doing the evaluation.
            Some(mut lock) => {
                match mem::replace(&mut *lock, EvaluationInProgress) {
                    Unevaluated(producer) => *lock = Evaluated(producer.invoke()),
                    // Since the OnceMutex only lets us get here once,
                    // it *must* contain Unevaluated.
                    _ => unsafe { debug_unreachable!() }
                }
            },
            // Already forced or forcing, so wait for the value
            // if we need to.
            //
            // Unfortunately, we do not know if this is a
            // recursive force, meaning this will cause a deadlock,
            // or if we are waiting on another thread.
            None => self.inner.wait()
        }
    }
}

impl<'a, T: Send + Sync> DerefMut for Thunk<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.force();
        match &mut *self.inner {
            &mut Evaluated(ref mut val) => val,

            // We just forced this thunk.
            _ => unsafe { debug_unreachable!() }
        }
    }
}

impl<'a,T: Send + Sync> Deref for Thunk<'a,T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.force();
        match *self.inner {
            Evaluated(ref val) => val,

            // We just forced this thunk.
            _ => unsafe { debug_unreachable!() }
        }
    }
}

struct Producer<'a,T> {
    inner: Box<FnBox<Output=T> + Send + Sync + 'a>
}

impl<'a,T> Producer<'a, T> {
    fn new<F: 'a + Send + Sync + FnOnce() -> T>(f: F) -> Producer<'a, T> {
        Producer { inner: Box::new(f) }
    }

    fn invoke(self) -> T {
        self.inner.call_box(())
    }
}

enum Inner<'a,T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(Producer<'a,T>)
}
