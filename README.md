# VSD-RS
Very Simple Database in Rust. Really, *very* simple.

VSD is a key-value in-memory database, which also periodically dumps its contents to disk.

Basically you can do this to write a value:
```
    extern crate vsd;
    use vsd::{VSD, VSDUnlocked};

    [..]

    let mut vsd = VSD::new();
    vsd.open("test_db.db");
    vsd.write::<u16>("test", 8u16);
```

And this to read a value:
```
    extern crate vsd;
    use vsd::{VSD, VSDUnlocked};

    [..]

    let mut vsd = VSD::new();
    vsd.open("test_db.db");
    assert_eq!(*(vsd.read::<u16>("test").unwrap()), 8);
```

If you don't want to write anything to disk, you can omit opening a database:
```
    extern crate vsd;
    use vsd::{VSD, VSDUnlocked};

    [..]

    let mut vsd = VSD::new();
    vsd.write::<String>("test", "Hello world!".to_string());
```

And if you prefer a caching system where writing to disk happens in its own thread, you simply bring the VSDLocked trait into scope instead of VSDUnlocked:
```
    extern crate vsd;
    use vsd::{VSD, VSDLocked};

    [..]

    let mut vsd = VSD::new();
    vsd.write::<String>("test", "Hello world!".to_string());
```

It should work over all types that are serializable by Serde, which covers all primitive types and many advanced types of the std.