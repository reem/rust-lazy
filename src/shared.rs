use std::cell::UnsafeCell;
use std::sync::RWLock;
use std::ptr;

/// A sometimes cleaner name.
pub type SharedLazy<T> = SharedThunk<T>;

/// Shareable, sendable lazy data.
pub struct SharedThunk<T> {
    inner: RWLock<UnsafeCell<SharedInner<T>>>
}

impl<T: Send> SharedThunk<T> {
    /// Create a new shared thunk.
    pub fn new(producer: proc(): Send -> T) -> SharedThunk<T> {
        SharedThunk { inner: RWLock::new(UnsafeCell::new(Unevaluated(producer))) }
    }

    /// Force evaluation of a thunk.
    pub fn force(&self) {
        unsafe {
            // Take out only a read lock.
            match *self.inner.read().get() {
                // Already forced. We're done and only took a read lock.
                Evaluated(_) => return,
                EvaluationInProgress => {
                    // Block until evaluation is over.
                    let _ = self.inner.write();

                    // Don't evaluate again.
                    return
                },

                // We have to evaluate the producer.
                Unevaluated(_) => ()
            }

            // Get a write lock for the entire evaluation period.
            let mut write_lock = self.inner.write();

            // Set the status to EvaluationInProgress
            match ptr::replace(write_lock.get(), EvaluationInProgress) {
                // Get the producer, evaluate it.
                Unevaluated(producer) => *write_lock.get() = Evaluated(producer()),
                _ => unreachable!()
            };
        }
    }
}

enum SharedInner<T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(proc(): Send -> T)
}

