extern crate vsd;

use vsd::{VSD, VSDLocked};
use std::thread;
use std::time::{Duration};

fn main() {
    let mut vsd = VSD::new();
    vsd.open("threaded_test.db");
    for i in 1..2
    {
        vsd.write::<usize>("write_test", i);
        thread::sleep(Duration::from_millis(1000));
    }
    println!("{:?}", vsd.read::<usize>("write_test").unwrap());
}