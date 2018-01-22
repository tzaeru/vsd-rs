extern crate vsd;

use vsd::{VSD, VSDUnlocked};

fn main() {
    let mut vsd = VSD::<f32, VSDUnlocked>::new();
    vsd.write("write_test", 0.5f32);
    println!("{:?}", vsd.read("write_test").unwrap());
}