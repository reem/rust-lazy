use oncemutex::OnceMutex;
use std::mem;
use std::thunk::Invoke;

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
    /// let expensive = Thunk::new(|| { println!("Evaluated!"); 7u });
    /// let reff = Arc::new(expensive);
    /// let reff_clone = reff.clone();
    ///
    /// // Evaluated is printed sometime beneath this line.
    /// spawn(move || {
    ///     assert_eq!(**reff_clone, 7u);
    /// });
    /// assert_eq!(**reff, 7u);
    /// ```
    pub fn new<F>(producer: F) -> Thunk<T>
    where F: Send + Sync + FnOnce() -> T {
        Thunk {
            inner: OnceMutex::new(Unevaluated(Producer::new(producer)))
        }
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

struct Producer<T> {
    inner: Box<Invoke<(), T> + Send + Sync>
}

impl<T> Producer<T> {
    fn new<F: Send + Sync + FnOnce() -> T>(f: F) -> Producer<T> {
        Producer {
            inner: box() (move |: ()| {
                f()
            }) as Box<Invoke<(), T> + Send + Sync>
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

