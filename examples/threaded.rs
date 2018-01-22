extern crate vsd;

use vsd::{VSD, VSDLocked};
use std::thread;
use std::time::{Duration};

fn main() {
    let mut vsd = VSD::<usize>::new();
    //vsd.open("basic_test.db");
    vsd.open("threaded_test.db");
    //vsd.write("write_test", 7);
    for i in 1..14
    {
        //vsd.write("write_test", i);
        //thread::sleep(Duration::from_millis(1000));
    }
    println!("{:?}", vsd.read("write_test").unwrap());
}