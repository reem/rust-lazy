#![feature(phase)]
#[phase(plugin, link)]
extern crate lazy;

use lazy::Thunk;

use std::sync::{Arc, Mutex};
use std::task;

#[test] fn test_evaluates() {
    let val = lazy!(7i);
    assert_eq!(*val, 7i);
}

#[test] fn test_evaluated() {
    let x = Thunk::evaluated(10u);
    assert_eq!(*x, 10u);
}

#[test] fn test_evaluates_just_once() {
    let counter = Arc::new(Mutex::new(0i));
    let counter_clone = counter.clone();
    let val = lazy!(*counter.lock() += 1);
    *val;
    *val;
    assert_eq!(*counter_clone.lock(), 1i);
}

#[test] fn test_doesnt_evaluate_if_not_accessed() {
    let counter = Arc::new(Mutex::new(0i));
    let counter_clone = counter.clone();
    let _val = lazy!(*counter.lock() += 1);
    assert_eq!(*counter_clone.lock(), 0i);
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

#[test] fn test_call_mutable_trait_methods() {
    let mut val = lazy!(7u);
    val.mutable_borrow_method();
}

#[test] fn test_unwrap() {
    let val = lazy!(7u);
    assert_eq!(val.unwrap(), 7u);
}

pub struct Dropper(Arc<Mutex<u64>>);

impl Drop for Dropper {
    fn drop(&mut self) {
        let Dropper(ref count) = *self;
        *count.lock() += 1;
        assert!(task::failing())
    }
}

#[test] fn test_calls_destructor_once() {
    let counter = Arc::new(Mutex::new(0u64));
    let counter_clone = counter.clone();
    match task::try(proc() {
        let value = Dropper(counter_clone);
        let t = Thunk::<()>::new(proc() {
            // Get a reference so value is captured.
            let _x = &value;

            fail!("Muahahahah")
        });
        t.force();
    }) {
        Err(_) => {
            assert_eq!(*counter.lock(), 1);
        },
        _ => ()
    }
}

