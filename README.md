# VSD-RS
Very Simple Database in Rust. Really, *very* simple.

VSD is a key-value in-memory database, which also periodically dumps its contents to disk.

WIP, still going to add examples.

But basically you can do this to write a value:
```
        let mut vsd = VSD::<u16>::new();
        vsd.open("test_db.db");
        vsd.write("test", 8u16);
```

And this to read a value:
```
        let mut vsd = VSD::<u16>::new();
        vsd.open("test_db.db");
        assert_eq!(*(vsd.read("test").unwrap()), 8);
```

If you don't want to write anything to disk, you can omit opening a database:
```
        let mut vsd = VSD::<u16>::new();
       	vsd.write("test", 8u16);
        assert_eq!(*(vsd.read("test").unwrap()), 8);
```

And if you prefer a caching system where writing to disk happens in its own thread, you can do:
```
        let mut vsd = VSD::<u16>::new();
        vsd.open_with_caching("test_db.db");
        vsd.write("test", 8u16);
```

It works over basically all standard primitive types and also various types in std.