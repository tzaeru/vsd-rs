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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, TryRecvError, Sender};

use std::fs;

use bincode::{serialize, deserialize, Infinite};

pub trait VSDUnlocked<T> {
    fn open(&mut self, file_name: &str);
    fn write(&mut self, key: &str, data: T);
    fn read(&self, key: &str) -> Option<&T>;
    fn flush(&mut self);
}
pub trait VSDLocked<T> {
    fn open(&mut self, file_name: &str);
    fn write(&mut self, key: &str, data: T);
    fn read(&self, key: &str) -> Option<T>;
}

pub struct VSD<T: 'static + serde::Serialize + serde::de::DeserializeOwned> {
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
    /// A sender to terminate the caching thread
    cache_terminate_sender: Option<Sender<()>>,
    /// Used to indicate whether we've flushed to disk after last write
    dirty: Arc<AtomicBool>,
    /// When we last wrote to the database
    last_written: Arc<Mutex<Instant>>,
    /// When we last flushed the database
    last_flushed: Arc<Mutex<Instant>>,
}

impl <T>VSD<T>
    where T: serde::Serialize + serde::de::DeserializeOwned
{
    pub fn new() -> VSD<T> {
        VSD {
            data: HashMap::new(),
            file: Err(io::Error::new(io::ErrorKind::NotFound, "Database file not opened")),
            data_locked: Arc::new(Mutex::new(HashMap::new())),
            file_locked: Arc::new(Mutex::new(Err(io::Error::new(io::ErrorKind::NotFound, "Database file not opened")))),
            caching_thread: None,
            cache_terminate_sender: None,
            dirty: Arc::new(AtomicBool::new(false)),
            last_written: Arc::new(Mutex::new(Instant::now())),
            last_flushed: Arc::new(Mutex::new(Instant::now()))
        }
    }
}

impl <T>VSDUnlocked<T> for VSD<T>
    where T: serde::Serialize + serde::de::DeserializeOwned {
    /// Opens and reads a database from file.
    /// If file doesn't exist, a new file for the db is created.
    fn open(&mut self, file_name: &str)
    {
        self.file = OpenOptions::new().read(true).append(false).write(true).create(true).open(file_name);

        let metadata = fs::metadata(file_name).unwrap();

        let mut decode_buf = Vec::new();

        if metadata.len() > 0
        {
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
    }

    /// Writes to database.
    fn write(&mut self, key: &str, data: T)
    {
        self.data.insert(key.to_string(), data);
        self.flush();
    }

    fn read(&self, key: &str) -> Option<&T>
    {
        for key in self.data.keys() {
            println!("{}", key);
        }
        return self.data.get(key);
    }

    fn flush(&mut self)
    {
        let encoded: Vec<u8> = serialize(&self.data, Infinite).unwrap();

        println!("Encoded length: {:?}", encoded.len());

        match self.file
        {
            // Must use references, or else ownership of File moves to match
            Ok(ref mut file) => {
                let _ = file.write_all(&encoded[..]);
            }
            Err(_) => {
                println!("Writing to file errored.");
            }
        }
    }
}

/*impl <T: serde::Serialize + serde::de::DeserializeOwned, LOCK: VSDUnlocked + ?Sized> VSD<T, LOCK> {



}*/

impl <T: serde::Serialize + serde::de::DeserializeOwned>Drop for VSD<T> {
    fn drop(&mut self) {
        println!("Dropping.");

        // Must match, since unwrap may consume the type if it panics.
        // Can't consume in Drop trait.
        match self.cache_terminate_sender
        {
            Some(ref sender) => {
                // Must match on sender or this function tries to return an inappropriate type? what?
                match sender.send(())
                {
                    Ok(()) => {},
                    Err(_) => {}
                }
            },
            None => {}
        }

        // TODO: Fix?
        // This is a little bit complicated and probably not the correct way to join.
        // Bad idea to join in Drop: https://stackoverflow.com/questions/41331577/joining-a-thread-in-a-method-that-takes-mut-self-like-drop-results-in-cann/42791007#42791007
        // But the idea here is that we can't move ownership around in Drop.
        // As such, we get the value held in the caching_thread Option by value by using the take() method.
        if let Some(handle) = self.caching_thread.take() {
            handle.join().expect("failed to join thread");
        }
    }
}
impl <T>VSDLocked<T> for VSD<T>
    where T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send {


    /// Opens and reads a database from file.
    /// If file doesn't exist, a new db is created.
    /// This will spawn a thread to periodically flush several writes to disk at once instead of flushing on every write
    fn open(&mut self, file_name: &str)
    {
        println!("Called the right open!");

        let mut file = OpenOptions::new().read(true).append(false).write(true).create(true).open(file_name);
        let metadata = fs::metadata(file_name).unwrap();
        let mut decode_buf = Vec::new();
        
        if metadata.len() > 0
        {
            match file
            {
                // Must use references, or else ownership of File moves to match
                Ok(ref mut file) => {
                    let _ = (*file).read_to_end(&mut decode_buf).unwrap();
                    self.data_locked = Arc::new(Mutex::new(deserialize(&decode_buf[..]).unwrap()));
                }
                Err(_) => {

                }
            }
        }

        self.file_locked = Arc::new(Mutex::new(file));

        //let local_self = Arc::new(self).clone();
        let data_clone = self.data_locked.clone();
        let file_clone = self.file_locked.clone();
        let dirty_clone = self.dirty.clone();
        let last_written_clone = self.last_written.clone();
        let last_flushed_clone = self.last_flushed.clone();
        let (sender, receiver) = mpsc::channel();
        self.cache_terminate_sender = Some(sender.clone());
        self.caching_thread = Some(thread::spawn(move || {
            let mut terminate = false;

            while !terminate {

                // We first check if we should terminate. If we should, we'll try to flush one more time.
                match receiver.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => {
                        terminate = true;
                    }
                    Err(TryRecvError::Empty) => {}
                }

                // If we're not dirty i.e. unflushed, sleep for a second, waiting to be dirty i.e. for there to be something to flush
                if dirty_clone.load(Ordering::Relaxed) == false
                {
                    thread::sleep(Duration::from_millis(1000));
                    continue;
                }

                // If:
                // a) it's been less than 2 seconds since last write and
                // b) it's been less than 10 seconds since last flush and
                // c) we're not terminating
                // we'll sleep for a second. We only flush to disc every 10-11 seconds or when it's been 2-3 seconds since write
                if last_written_clone.lock().unwrap().elapsed().as_secs() < 2
                   && last_flushed_clone.lock().unwrap().elapsed().as_secs() < 10
                   && !terminate
                {
                    thread::sleep(Duration::from_millis(1000));
                    continue;
                }

                println!("We're writing data!");

                let data = data_clone.lock().unwrap();
                let encoded: Vec<u8> = serialize(&(*data), Infinite).unwrap();
                println!("{:?}", encoded[0]);

                // Must do the name bind or else we'll have a bug here.
                let mut file_lock = file_clone.lock().unwrap();

                match *file_lock
                {
                    Ok(ref mut file) => {
                        let _ = file.write_all(&encoded[..]);
                        dirty_clone.store(false, Ordering::Relaxed);
                        let mut last_flushed = last_flushed_clone.lock().unwrap();
                        *last_flushed = Instant::now();
                    }
                    Err(_) => {

                    }
                }
            }
        }));
    }

    /// Writes to database.
    fn write(&mut self, key: &str, data: T)
    {
        self.data_locked.lock().unwrap().insert(key.to_string(), data);
        self.dirty.store(true, Ordering::Relaxed);
        let mut last_written = self.last_written.lock().unwrap();
        *last_written = Instant::now();
    }

    fn read(&self, key: &str) -> Option<T>
    {
        // We can't return directly, see: https://stackoverflow.com/a/32083561
        for key in self.data_locked.lock().unwrap().keys() {
            println!("{}", key);
        }
        let data = self.data_locked.lock().unwrap();
        let value = data.get(key).unwrap();
        let clone = value.clone();
        return Some(clone);
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
