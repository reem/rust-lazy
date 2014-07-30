use super::Thunk;

use std::sync::{Arc, Mutex};

#[test]
fn test_evaluates() {
    let val = lazy!(7i);
    assert_eq!(*val, 7i);
}

#[test]
fn test_evaluates_just_once() {
    let counter = Arc::new(Mutex::new(0i));
    let counter_clone = counter.clone();
    let val = lazy!(*counter.lock() += 1);
    *val;
    *val;
    assert_eq!(*counter_clone.lock(), 1i);
}

//trait ImmutableReferenceTrait {
//    fn borrow_method(&self) { () }
//}
//
//impl ImmutableReferenceTrait for uint {}
//
//#[test]
//fn test_call_trait_methods() {
//    let val = lazy!(7u);
//    val.borrow_method();
//}

trait MutableReferenceTrait {
    fn mutable_borrow_method(&mut self) { () }
}

impl MutableReferenceTrait for uint {}

#[test]
fn test_call_mutable_trait_methods() {
    let mut val = lazy!(7u);
    val.mutable_borrow_method();
}

