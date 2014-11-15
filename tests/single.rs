#![feature(phase)]
#[phase(plugin, link)]
extern crate lazy;
#[phase(plugin)]
extern crate stainless;

pub use lazy::Thunk;
pub use std::sync::{Arc, Mutex};
pub use std::task;

describe! thunk {
    it "should evaluate when accessed" {
        let val = lazy!(7i);
        assert_eq!(*val, 7i);
    }

    it "should evaluate just once" {
        let counter = Arc::new(Mutex::new(0i));
        let counter_clone = counter.clone();
        let val = lazy!(*counter.lock() += 1);
        *val;
        *val;
        assert_eq!(*counter_clone.lock(), 1i);
    }

    it "should not evaluate if not accessed" {
        let counter = Arc::new(Mutex::new(0i));
        let counter_clone = counter.clone();
        let _val = lazy!(*counter.lock() += 1);
        assert_eq!(*counter_clone.lock(), 0i);
    }

    describe! methods {
        it "should allow mutable trait methods" {
            let mut val = lazy!(7u);
            val.mutable_borrow_method();
        }
    }

    describe! unwrap {
        it "should retrieve the value" {
            let val = lazy!(7u);
            assert_eq!(val.unwrap(), 7u);
        }
    }

    describe! evaluated {
        it "should produce an already evaluated thunk" {
            let x = Thunk::evaluated(10u);
            assert_eq!(*x, 10u);
        }
    }

    describe! drop {
        it "should drop internal data just once" {
            let counter = Arc::new(Mutex::new(0u64));
            let counter_clone = counter.clone();
            match task::try(proc() {
                let value = Dropper(counter_clone);
                let t = Thunk::<()>::new(proc() {
                    // Get a reference so value is captured.
                    let _x = &value;

                    panic!("Muahahahah")
                });
                t.force();
            }) {
                Err(_) => {
                    assert_eq!(*counter.lock(), 1);
                },
                _ => panic!("Unexpected success in spawned task.")
            }
        }
    }
}

pub trait ImmutableReferenceTrait {
    fn borrow_method(&self) { () }
}

impl ImmutableReferenceTrait for uint {}

#[test]
fn test_call_trait_methods() {
    let val = lazy!(7u);
    val.borrow_method();
}

pub trait MutableReferenceTrait {
    fn mutable_borrow_method(&mut self) { () }
}

impl MutableReferenceTrait for uint {}

pub struct Dropper(Arc<Mutex<u64>>);

impl Drop for Dropper {
    fn drop(&mut self) {
        let Dropper(ref count) = *self;
        *count.lock() += 1;
        assert!(task::failing())
    }
}

