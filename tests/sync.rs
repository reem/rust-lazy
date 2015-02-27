#![feature(plugin, std_misc, old_io)]
#![plugin(stainless)]

#[macro_use]
extern crate lazy;

pub use lazy::sync::Thunk;
pub use std::sync::{Arc, Barrier, Mutex};
pub use std::{old_io, time};
pub use std::thread;

describe! sync {
    it "should evaluate when accessed" {
        let val = sync_lazy!(7);
        assert_eq!(*val, 7);
    }

    it "should evaluate just once" {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let val = sync_lazy!({
            let mut data = counter.lock().unwrap();
            *data += 1;
        });
        *val;
        *val;
        assert_eq!(*counter_clone.lock().unwrap(), 1);
    }

    it "should not evaluate if not accessed" {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let _val = sync_lazy!({
            let mut data = counter.lock().unwrap();
            *data += 1;
        });
        assert_eq!(*counter_clone.lock().unwrap(), 0);
    }

    it "should be send and sync" {
        Arc::new(sync_lazy!(0));
    }

    it "should be safe to access while evaluating" {
        let data = Arc::new(sync_lazy!({
            old_io::timer::sleep(time::Duration::milliseconds(50));
            5
        }));

        let data_worker = data.clone();

        // Worker task.
        thread::spawn(move || {
            data_worker.force();
        });

        // Try to access the data while it is evaulating.
        assert_eq!(5, **data);
    }

    describe! evaluated {
        it "should create an already evaluated thunk" {
            let x = Thunk::evaluated(10);
            assert_eq!(*x, 10);
        }
    }
}
