use oncemutex::OnceMutex;
use std::mem;

use self::Inner::{Evaluated, EvaluationInProgress, Unevaluated};

/// A sometimes cleaner name.
pub type Lazy<T> = Thunk<T>;

/// Sync, Send lazy data.
pub struct Thunk<T> {
    inner: OnceMutex<Inner<T>>
}

impl<T: Send + Sync> Thunk<T> {
    /// Create a new sync thunk.
    ///
    /// You can construct Thunk's manually using this, but the
    /// sync_lazy! macro is preferred.
    ///
    /// ```rust
    /// # use lazy::sync::Thunk;
    /// # use std::sync::Arc;
    /// let expensive = Thunk::new(proc() { println!("Evaluated!"); 7u });
    /// let reff = Arc::new(expensive);
    /// let reff_clone = reff.clone();
    ///
    /// // Evaluated is printed sometime beneath this line.
    /// spawn(proc() {
    ///     assert_eq!(**reff_clone, 7u);
    /// });
    /// assert_eq!(**reff, 7u);
    /// ```
    pub fn new(producer: proc(): Send + Sync -> T) -> Thunk<T> {
        Thunk { inner: OnceMutex::new(Unevaluated(producer)) }
    }

    /// Create a new, evaluated, thunk from a value.
    pub fn evaluated(val: T) -> Thunk<T> {
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
                    Unevaluated(producer) => *lock = Evaluated(producer()),
                    // Since the OnceMutex only lets us get here once,
                    // it *must* contain Unevaluated.
                    _ => unsafe { debug_unreachable!() }
                }
            },
            // Already forced, do nothing.
            None => {}
        }
    }
}

impl<T: Send + Sync> DerefMut<T> for Thunk<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.force();
        match *&mut*self.inner {
            // Safe because getting this &'a mut T requires &'a mut self.
            //
            // We can't use copy_mut_lifetime here because self is already
            // borrowed as &mut by val.
            Evaluated(ref mut val) => unsafe { mem::transmute(val) },

            // We just forced this thunk.
            _ => unsafe { debug_unreachable!() }
        }
    }
}

impl<T: Send + Sync> Deref<T> for Thunk<T> {
    fn deref(&self) -> &T {
        self.force();
        match *self.inner {
            // Safe because getting this &'a T requires &'a self.
            Evaluated(ref val) => unsafe { mem::copy_lifetime(self, val) },

            // We just forced this thunk.
            _ => unsafe { debug_unreachable!() }
        }
    }
}

enum Inner<T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(proc(): Send + Sync -> T)
}

