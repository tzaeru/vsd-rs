# VSD-RS
Very Simple Database in Rust. Really, *very* simple.

WIP, still going to add examples.

But basically you can do this:

```
        let mut vsd = VSD::<u16>::new();
        vsd.open("test_db.db");
        vsd.write("test", 8u16);
```

```
        let mut vsd = VSD::<u16>::new();
        vsd.open("test_db.db");
        assert_eq!(*(vsd.read("test").unwrap()), 8);
```

It works over basically all standard primitive types and also various types in std.