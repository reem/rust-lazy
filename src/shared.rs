use std::sync::RWLock;
use std::mem;

/// A sometimes cleaner name.
pub type SharedLazy<T> = SharedThunk<T>;

/// Shareable, sendable lazy data.
pub struct SharedThunk<T> {
    inner: RWLock<SharedInner<T>>
}

impl<T: Send + Share> SharedThunk<T> {
    /// Create a new shared thunk.
    pub fn new(producer: proc(): Send + Share -> T) -> SharedThunk<T> {
        SharedThunk { inner: RWLock::new(Unevaluated(producer)) }
    }

    /// Force evaluation of a thunk.
    pub fn force(&self) {
        // Take out only a read lock.
        match *self.inner.read() {
            // Already forced. We're done and only took a read lock.
            Evaluated(_) => return,
            EvaluationInProgress => unreachable!(),
            // First ones here. Evaluate.
            Unevaluated(_) => ()
        }

        // Get a write lock for the entire evaluation period.
        let mut write_lock = self.inner.write();

        // Set the status to EvaluationInProgress
        match mem::replace(&mut *write_lock, EvaluationInProgress) {
            // Get the producer, evaluate it.
            Unevaluated(producer) => *write_lock = Evaluated(producer()),
            _ => unreachable!()
        };
    }
}

        }
    }
}

enum SharedInner<T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(proc(): Send + Share -> T)
}

