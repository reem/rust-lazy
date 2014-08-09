use super::SyncThunk;

use std::sync::{Arc, Mutex};

#[test] fn test_evaluates() {
    let val = sync_lazy!(7i);
    assert_eq!(*val, 7i);
}

#[test] fn test_evaluates_just_once() {
    let counter = Arc::new(Mutex::new(0i));
    let counter_clone = counter.clone();
    let val = sync_lazy!(*counter.lock() += 1);
    *val;
    *val;
    assert_eq!(*counter_clone.lock(), 1i);
}

#[test] fn test_doesnt_evaluate_if_not_accessed() {
    let counter = Arc::new(Mutex::new(0i));
    let counter_clone = counter.clone();
    let _val = sync_lazy!(*counter.lock() += 1);
    assert_eq!(*counter_clone.lock(), 0i);
}

#[test] fn test_is_sync_and_send() {
    let _ = Arc::new(sync_lazy!(0u));
}

#[test] fn test_evaluated() {
    let x = SyncThunk::evaluated(10u);
    assert_eq!(*x, 10u);
}

// sync is all safe code, so no need to test
// destructor calls.

