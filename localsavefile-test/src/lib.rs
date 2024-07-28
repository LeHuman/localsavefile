#![cfg(test)]
use localsavefile::{localsavefile, LocalSaveFile, LocalSaveFileCommon, LocalSaveFilePersistent};
use tracing::{debug, Level};
use tracing_subscriber::FmtSubscriber;

#[test]
fn test_trait() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    #[localsavefile]
    #[derive(Default)]
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
    #[derive(Default)]
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
    #[derive(Debug, PartialEq, Default)]
    struct TestCache {
        val: u32,
        #[savefile_default_val = "blank"]
        #[savefile_versions = "0..0"]
        str: String,
    }

    #[localsavefile(persist = true, name = "some_savefile")]
    #[derive(Debug, PartialEq, Default)]
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
