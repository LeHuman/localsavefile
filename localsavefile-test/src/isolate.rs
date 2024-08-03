use localsavefile::{localsavefile, LocalSaveFile, LocalSaveFileCommon};
#[test]
fn test_isolated() {
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
}
