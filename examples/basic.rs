extern crate vsd;

use vsd::{VSD, VSDUnlocked};

fn main() {
    let mut vsd = VSD::new();
    vsd.open("basic_test.db");
    vsd.write::<u8>("write_test", 7);
    println!("{:?}", vsd.read::<u8>("write_test").unwrap());
}