use std::sync::RWLock;
use std::mem;

/// A sometimes cleaner name.
pub type SyncLazy<T> = SyncThunk<T>;

/// Syncable, sendable lazy data.
pub struct SyncThunk<T> {
    inner: RWLock<SyncInner<T>>
}

impl<T: Send + Sync> SyncThunk<T> {
    /// Create a new shared thunk.
    pub fn new(producer: proc(): Send + Sync -> T) -> SyncThunk<T> {
        SyncThunk { inner: RWLock::new(Unevaluated(producer)) }
    }

    /// Create a new, evaluated, thunk from a value.
    pub fn evaluated(val: T) -> SyncThunk<T> {
        SyncThunk { inner: RWLock::new(Evaluated(val)) }
    }

    /// Force evaluation of a thunk.
    pub fn force(&self) {
        // Take out only a read lock.
        match *self.inner.read() {
            // Already forced. We're done and only took a read lock.
            Evaluated(_) => return,

            // Can't happen because this requires someone else to have
            // a write lock at the same time.
            EvaluationInProgress => unreachable!(),

            // First ones here. Evaluate.
            Unevaluated(_) => ()
        }

        // Get a write lock for the entire evaluation period.
        let mut write_lock = self.inner.write();

        match *write_lock {
            // If two threads try to call force at the same time,
            // then the write locks may be queued up and inner may
            // have been evaluated already.
            Evaluated(_) => return,
            _ => ()
        }

        // Set the status to EvaluationInProgress
        match mem::replace(&mut *write_lock, EvaluationInProgress) {
            // Get the producer, evaluate it.
            Unevaluated(producer) => *write_lock = Evaluated(producer()),

            // We checked these possibilities earlier.
            _ => unreachable!()
        };
    }
}

impl<T: Send + Sync> DerefMut<T> for SyncThunk<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.force();
        match &mut *self.inner.write() {
            // Safe because getting this &'a mut T requires &'a mut self.
            //
            // We can't use copy_mut_lifetime here because self is already
            // borrowed as &mut by self.inner.write().
            &Evaluated(ref mut val) => unsafe { mem::transmute(val) },

            // We just forced this thunk.
            _ => unreachable!()
        }
    }
}

impl<T: Send + Sync> Deref<T> for SyncThunk<T> {
    fn deref(&self) -> &T {
        self.force();
        match *self.inner.read() {
            // Safe because getting this &'a T requires &'a self.
            Evaluated(ref val) => unsafe { mem::copy_lifetime(self, val) },

            // We just forced this thunk.
            _ => unreachable!()
        }
    }
}

enum SyncInner<T> {
    Evaluated(T),
    EvaluationInProgress,
    Unevaluated(proc(): Send + Sync -> T)
}

