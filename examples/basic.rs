extern crate vsd;

use vsd::{VSD, VSDUnlocked};

fn main() {
    let mut vsd = VSD::<usize>::new();
    vsd.open("basic_test.db");
    vsd.write("write_test", 9);
    println!("{:?}", vsd.read("write_test").unwrap());
}