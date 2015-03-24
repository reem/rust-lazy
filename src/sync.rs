use std::ops::{Deref, DerefMut};
use oncemutex::OnceMutex;
use std::mem;
use std::thunk::Invoke;

use self::Inner::{Evaluated, EvaluationInProgress, Unevaluated};

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
    /// # use std::thread::Thread;
    /// let expensive = Thunk::new(|| { println!("Evaluated!"); 7u });
    /// let reff = Arc::new(expensive);
    /// let reff_clone = reff.clone();
    ///
    /// // Evaluated is printed sometime beneath this line.
    /// Thread::spawn(move || {
    ///     assert_eq!(**reff_clone, 7u);
    /// });
    /// assert_eq!(**reff, 7u);
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

impl<'a,T: Send + Sync> Deref for Thunk<'a,T> {
    type Target = T;

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

struct Producer<'a,T> {
    inner: Box<Invoke<(), T> + Send + Sync + 'a>
}

impl<'a,T> Producer<'a, T> {
    fn new<F: 'a + Send + Sync + FnOnce() -> T>(f: F) -> Producer<'a, T> {
        Producer {
            inner: Box::new(move |()| {
                f()
            }) as Box<Invoke<(), T> + Send + Sync>
        }
    }

    fn invoke(self) -> T {
        self.inner.invoke(())
    }
}

enum Inner<'a,T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(Producer<'a,T>)
}
