extern crate vsd;

use vsd::{VSD, VSDUnlocked};

fn main() {
    let mut vsd = VSD::new();
    vsd.write::<f32>("test_float_value", 0.5f32);
    println!("{:?}", vsd.read::<f32>("test_float_value").unwrap());
}