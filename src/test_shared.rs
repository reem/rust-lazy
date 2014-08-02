use super::SharedThunk;

use std::sync::{Arc, Mutex};

#[test] fn test_evaluates() {
    let val = shared_lazy!(7i);
    assert_eq!(*val, 7i);
}

#[test] fn test_evaluates_just_once() {
    let counter = Arc::new(Mutex::new(0i));
    let counter_clone = counter.clone();
    let val = shared_lazy!(*counter.lock() += 1);
    *val;
    *val;
    assert_eq!(*counter_clone.lock(), 1i);
}

#[test] fn test_doesnt_evaluate_if_not_accessed() {
    let counter = Arc::new(Mutex::new(0i));
    let counter_clone = counter.clone();
    let _val = shared_lazy!(*counter.lock() += 1);
    assert_eq!(*counter_clone.lock(), 0i);
}

#[test] fn test_is_shared_and_send() {
    let _ = Arc::new(shared_lazy!(0u));
}

#[test] fn test_evaluated() {
    let x = SharedThunk::evaluated(10u);
    assert_eq!(*x, 10u);
}

// Shared is all safe code, so no need to test
// destructor calls.

