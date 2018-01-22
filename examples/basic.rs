extern crate vsd;

use vsd::{VSD, VSDUnlocked};

fn main() {
    let mut vsd = VSD::<String, VSDUnlocked>::new();
    vsd.open("basic_test.db");
    vsd.write("write_test", "hello world!".to_string());
    println!("{:?}", vsd.read("write_test").unwrap());
}