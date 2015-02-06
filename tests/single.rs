#![feature(plugin, std_misc)]

#[macro_use]
extern crate lazy;

#[plugin]
extern crate stainless;

pub use lazy::single::Thunk;
pub use std::sync::{Arc, Mutex};
pub use std::thread::{self, Thread};

describe! thunk {
    it "should evaluate when accessed" {
        let val = lazy!(7);
        assert_eq!(*val, 7);
    }

    it "should evaluate just once" {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let val = lazy!(*counter.lock().unwrap() += 1);
        *val;
        *val;
        assert_eq!(*counter_clone.lock().unwrap(), 1);
    }

    it "should not evaluate if not accessed" {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let _val = lazy!(*counter.lock().unwrap() += 1);
        assert_eq!(*counter_clone.lock().unwrap(), 0);
    }

    describe! unwrap {
        it "should retrieve the value" {
            let val = lazy!(7);
            assert_eq!(val.unwrap(), 7);
        }
    }

    describe! evaluated {
        it "should produce an already evaluated thunk" {
            let x = Thunk::evaluated(10);
            assert_eq!(*x, 10);
        }
    }

    describe! drop {
        it "should drop internal data just once" {
            let counter = Arc::new(Mutex::new(0));
            let counter_clone = counter.clone();
            let result =  Thread::scoped(move || {
                let value = Dropper(counter_clone);
                let t = Thunk::<()>::new(move || {
                    // Get a reference so value is captured.
                    let _x = &value;

                    panic!("Muahahahah")
                });
                t.force();
            }).join();

            match result {
                Err(_) => {
                    assert_eq!(*counter.lock().unwrap(), 1);
                },
                _ => panic!("Unexpected success in spawned task.")
            }
        }
    }
}

pub struct Dropper(Arc<Mutex<u64>>);

impl Drop for Dropper {
    fn drop(&mut self) {
        let Dropper(ref count) = *self;
        *count.lock().unwrap() += 1;
    }
}

