#[macro_use]
extern crate bincode;
extern crate serde;

use std::collections::HashMap;
use std::option::Option;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::Read;
use std::fs::metadata;

use bincode::{serialize, deserialize, Infinite};

pub struct VSD<T: serde::Serialize + serde::de::DeserializeOwned> {
	data: HashMap<String, T>,
	file: String
}

impl <T: serde::Serialize + serde::de::DeserializeOwned> VSD<T> {
    pub fn new() -> VSD<T> {
        VSD {
        	data: HashMap::new(),
        	file: "".to_string()
        }
    }

	/// Opens and reads a database from file.
	/// If file doesn't exist, a new db is created.
	pub fn open(&mut self, file_name: &str)
	{
		let mut file = OpenOptions::new().read(true).write(true).create(true).open(file_name).unwrap();
		//let md = metadata(file_name).unwrap();

		// Unfortunately read_to_end fails if there's nothing to read, i.e. if the file is empty.
		// Soo we do this ugly hack to check if the file is larger than 0 bytes in size.

		let mut decode_buf = Vec::new();
		let to_read = file.read_to_end(&mut decode_buf).unwrap();
		self.data = deserialize(&decode_buf[..]).unwrap();

		self.file = file_name.to_string();
	}
	/// Opens and reads a database from file.
	/// If file doesn't exist, a new db is created.
	/// This will spawn a thread to periodically flush several writes to disk at once instead of every write flushing.
	pub fn open_with_caching(&self, file_name: &str)
	{

	}

	/// Writes to database.
	pub fn write(&mut self, key: &str, data: T)
	{
		self.data.insert(key.to_string(), data);
		self.flush();
	}

	pub fn read(&self, key: &str) -> Option<&T>
	{
		return self.data.get(key);
	}

	fn flush(&self)
	{
		let encoded: Vec<u8> = serialize(&self.data, Infinite).unwrap();
		let mut file = OpenOptions::new().write(true).create(true).open(&self.file).unwrap();
		file.write_all(&encoded[..]);
	}
}

/*impl VSD<i8> {
	fn flush(&self)
	{
		let encoded: Vec<u8> = serialize(&self.data, Infinite).unwrap();
	}
}*/

#[cfg(test)]
mod tests {
	use super::*;

    #[test]
    fn test_writing_reading_integers() {
        let mut vsd = VSD::<u16>::new();
        vsd.open("test1");
        vsd.write("write_test", 8u16);
        // Test for a true value
		assert_eq!(*(vsd.read("write_test").unwrap()), 8u16);
		// Test for a false value
		assert_ne!(*(vsd.read("write_test").unwrap()), 10u16);
    }

    #[test]
    fn test_writing_reading_strings() {
        let mut vsd = VSD::<String>::new();
        vsd.open("test2");
        vsd.write("write_test", "hello".to_string());
        // Test for a true value
		assert_eq!(*(vsd.read("write_test").unwrap()), "hello".to_string());
		// Test for a false value
		assert_ne!(*(vsd.read("write_test").unwrap()), "goodbye".to_string());
		
		vsd = VSD::<String>::new();
		vsd.open("test2");
		// Test for a true value
		assert_eq!(*(vsd.read("write_test").unwrap()), "hello".to_string());
		// Test for a false value
		assert_ne!(*(vsd.read("write_test").unwrap()), "goodbye".to_string());
    }
}
