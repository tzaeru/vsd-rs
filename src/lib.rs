#[macro_use]
extern crate bincode;
extern crate serde;

use std::collections::HashMap;
use std::option::Option;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::Read;
use std::fs::File;
use std::io;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::marker::PhantomData;

use bincode::{serialize, deserialize, Infinite};

pub trait VSDLocked {}
pub trait VSDUnlocked {}

// Inner thing. Can we avoid using this?
/*
pub struct VSD<'a, T: 'a + serde::Serialize + serde::de::DeserializeOwned, LOCK> {
	inner: &'a mut Arc<VSDInner<T, LOCK>>
}*/

struct VSD<T: 'static + serde::Serialize + serde::de::DeserializeOwned, LOCK> {
	/// Sync data structure used when VSD operates in a non-locking mode
	data: HashMap<String, T>,
	/// Sync handle to database file used when VSD operates in a non-locking mode
	file: io::Result<File>,
	/// Mutex'd data structure used when VSD operates in a locking mode
	data_locked: Arc<Mutex<HashMap<String, T>>>,
	/// Mutex'd handle to database file used when VSD operates in a locking mode
	file_locked: Arc<Mutex<io::Result<File>>>,
	/// JoinHandle for a caching thread used to flush periodically to disk
	caching_thread: Option<thread::JoinHandle<()>>,
	/// Used to indicate whether we've flushed to disk after last write
	dirty: AtomicBool,
	/// When we last wrote to the database
	last_written: Arc<Mutex<Instant>>,
	/// When we last flushed the database
	last_flushed: Arc<Mutex<Instant>>,
	/// Used to get rid of "parameter 'LOCK' is never used" error from compiler.
	phantom_lock: PhantomData<LOCK>,
}

impl <T: serde::Serialize + serde::de::DeserializeOwned, LOCK: VSDLocked> VSD<T, LOCK> {
    pub fn new() -> VSD<T, LOCK> {
        VSD {
        	data: HashMap::new(),
        	file: Err(io::Error::new(io::ErrorKind::NotFound, "Database file not opened")),
        	data_locked: Arc::new(Mutex::new(HashMap::new())),
        	file_locked: Arc::new(Mutex::new(Err(io::Error::new(io::ErrorKind::NotFound, "Database file not opened")))),
        	caching_thread: None,
        	dirty: AtomicBool::new(false),
        	last_written: Arc::new(Mutex::new(Instant::now())),
			last_flushed: Arc::new(Mutex::new(Instant::now())),
			phantom_lock: PhantomData
        }
    }

	/// Opens and reads a database from file.
	/// If file doesn't exist, a new db is created.
	pub fn open(&mut self, file_name: &str)
	{
		self.file = OpenOptions::new().read(true).write(true).create(true).open(file_name);

		let mut decode_buf = Vec::new();
		match self.file
		{
			// Must use references, or else ownership of File moves to match
			Ok(ref mut file) => {
				let _ = (*file).read_to_end(&mut decode_buf).unwrap();
				self.data = deserialize(&decode_buf[..]).unwrap();
			}
			Err(_) => {

			}
		}
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

	fn flush(&mut self)
	{
		let encoded: Vec<u8> = serialize(&self.data, Infinite).unwrap();

		match self.file
		{
			// Must use references, or else ownership of File moves to match
			Ok(ref mut file) => {
				file.write_all(&encoded[..]);
			}
			Err(_) => {

			}
		}
	}
}

impl <T: serde::Serialize + serde::de::DeserializeOwned, VSDLocked>Drop for VSD<T, VSDLocked> {
	fn drop(&mut self) {
        println!("Dropping.");
    }
}

impl <T: serde::Serialize + serde::de::DeserializeOwned + Send + Sync, LOCK: VSDLocked + Send> VSD<T, LOCK> {
	/// Opens and reads a database from file.
	/// If file doesn't exist, a new db is created.
	/// This will spawn a thread to periodically flush several writes to disk at once instead of flushing on every write
	pub fn open_with_caching(&mut self, file_name: &str)
	{
		self.file_locked = Arc::new(Mutex::new(OpenOptions::new().read(true).write(true).create(true).open(file_name)));

		let mut decode_buf = Vec::new();
		match self.file
		{
			// Must use references, or else ownership of File moves to match
			Ok(ref mut file) => {
				let _ = (*file).read_to_end(&mut decode_buf).unwrap();
				self.data = deserialize(&decode_buf[..]).unwrap();
			}
			Err(_) => {

			}
		}

		//let local_self = Arc::new(self).clone();
		let data_clone = self.data_locked.clone();
		let file_clone = self.file_locked.clone();
		self.caching_thread = Some(thread::spawn(move || {
			let data = data_clone.lock().unwrap();
			let encoded: Vec<u8> = serialize(&(*data), Infinite).unwrap();
			println!("{:?}", encoded[0]);

			// Must do the name bind or else we'll have a bug here.
			let mut file_lock = file_clone.lock().unwrap();

			match *file_lock
			{
				Ok(ref mut file) => {
					file.write_all(&encoded[..]);
				}
				Err(_) => {

				}
			}
		}));
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
        let mut vsd = VSD::<u16, VSDUnlocked>::new();
        vsd.open("test1");
        vsd.write("write_test", 8u16);
        // Test for a true value
		assert_eq!(*(vsd.read("write_test").unwrap()), 8u16);
		// Test for a false value
		assert_ne!(*(vsd.read("write_test").unwrap()), 10u16);
    }

    #[test]
    fn test_writing_reading_strings() {
        let mut vsd = VSD::<String, VSDUnlocked>::new();
        vsd.open("test2");
        vsd.write("write_test", "hello".to_string());
        // Test for a true value
		assert_eq!(*(vsd.read("write_test").unwrap()), "hello".to_string());
		// Test for a false value
		assert_ne!(*(vsd.read("write_test").unwrap()), "goodbye".to_string());
		
		vsd = VSD::<String, VSDUnlocked>::new();
		vsd.open("test2");
		// Test for a true value
		assert_eq!(*(vsd.read("write_test").unwrap()), "hello".to_string());
		// Test for a false value
		assert_ne!(*(vsd.read("write_test").unwrap()), "goodbye".to_string());
    }
}
