#![cfg(test)]

use localsavefile::{
    localsavefile, localsavefile_impl, LocalSaveFile, LocalSaveFileCommon, LocalSaveFilePersistent,
};
use savefile::prelude::Savefile;
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;

pub mod isolate;

#[test]
fn test_trait() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    #[localsavefile]
    struct MySave {
        val: u32,
    }

    let foo = MySave { val: 21 };
    foo.save().unwrap();
    let bar = MySave::load_default();
    let mut baz = MySave { val: 42 };
    baz.load().unwrap();
    assert_eq!(foo.val, bar.val); // Should never trigger
    assert_eq!(bar.val, baz.val); // Should never trigger
    MySave::remove_file().unwrap();

    #[localsavefile(persist = true)]
    struct MySavePersist {
        val: u32,
    }

    let mut foo = MySavePersist {
        val: 21,
        // If you must create an inline struct, make use of your IDE to auto fill the following
        __place_localsavefile_above_any_derives: Default::default(),
    };
    foo.save().unwrap(); // Save now requires foo to be mutable
    let mut bar = MySavePersist::load_default();
    assert_eq!(foo.val, bar.val); // Should never trigger
    foo.close();
    bar.close(); // Requires bar to be mutable
                 // Close any instances before removing the file
    MySavePersist::remove_file().unwrap();

    #[localsavefile(version = 1)]
    #[derive(Debug, PartialEq)]
    struct TestCache {
        val: u32,
        #[savefile_default_val = "blank"]
        #[savefile_versions = "0..0"]
        str: String,
    }

    #[derive(Debug, PartialEq)]
    #[localsavefile_impl(persist = true, name = "some_savefile")]
    #[derive(Savefile, Default)]
    struct TestCache2 {
        val: u32,
        str: String,
    }

    let mut c: TestCache2 = TestCache2::load_default();
    c.val = 5;
    c.str = "😎😋😊".to_string();
    c.save().expect("Failed to save c");
    let mut d = TestCache2 {
        val: 2,
        str: "Not the same!".to_string(),
        __place_localsavefile_above_any_derives: Default::default(),
    };
    d.load().expect("Failed to load d");
    assert_eq!(c, d, "c != d");
    TestCache2::remove_file().expect("Failed to remove file");

    let mut a = TestCache {
        val: 16,
        str: "not blank".to_string(),
    };
    debug!("Version {}", TestCache::get_version());

    a.save().expect("Failed to save a");

    let mut b = TestCache::load_default();
    assert_eq!(b.val, a.val, "val should match");
    assert_ne!(b.str, a.str, "str should differ due to versioning");

    b.val = 8;
    b.save().expect("Failed to save b");
    a.load().expect("Failed to load a");
    assert_eq!(b, a, "a != b");
    TestCache::remove_file().expect("Failed to remove file");
}

#[test]
fn test_template() {
    #[localsavefile(persist = true)]
    #[derive(PartialEq, Debug)]
    struct MySaveT0<T>
    where
        T: Clone,
    {
        val: T,
    }

    type MySave0 = MySaveT0<u128>;
    let mut foo = MySave0 {
        val: 21,
        __place_localsavefile_above_any_derives: Default::default(),
    };
    foo.save().unwrap();
    let bar = MySave0::load_default();
    let mut baz = MySave0 {
        val: 42,
        __place_localsavefile_above_any_derives: Default::default(),
    };
    baz.load().unwrap();
    assert_eq!(foo, bar); // Should never trigger
    MySave0::remove_file().unwrap();

    #[localsavefile]
    #[derive(PartialEq, Debug)]
    struct MySaveT1<T, D, A, C> {
        val0: T,
        val1: D,
        val2: A,
        val3: A,
        val4: C,
    }

    type MySave1 = MySaveT1<u128, String, bool, MySave0>;
    let foo = MySave1 {
        val0: 16,
        val1: String::from("heyo"),
        val2: false,
        val3: true,
        val4: MySave0 {
            val: 21,
            __place_localsavefile_above_any_derives: Default::default(),
        },
    };
    foo.save().unwrap();
    let bar = MySave1::load_default();
    let mut baz = MySave1::default();
    baz.load().unwrap();
    assert_eq!(foo, bar); // Should never trigger
    MySave1::remove_file().unwrap();
}
