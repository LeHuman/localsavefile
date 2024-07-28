<!-- PROJECT: localsavefile -->
<!-- TITLE: localsavefile -->
<!-- KEYWORDS: library  -->
<!-- LANGUAGES: Rust -->
<!-- STATUS: Work In Progress -->

# LocalSaveFile

[![Crates.io](https://img.shields.io/crates/v/localsavefile.svg)](https://crates.io/crates/localsavefile)
[![Docs.rs](https://docs.rs/localsavefile/badge.svg)](https://docs.rs/localsavefile)
[![CI](https://github.com/lehuman/localsavefile/workflows/CI/badge.svg)](https://github.com/lehuman/localsavefile/actions)

[About](#about) - [Usage](#usage) - [Related](#related) - [License](#license) - [Contribution](#contribution)

## Status

**`Work In progress`**

## About
<!-- DESCRIPTION START -->
Save and load structs from a local file. A convenience wrapper around the [savefile](https://github.com/avl/savefile) crate.
<!-- DESCRIPTION END -->

LocalSaveFile takes care of where and how a `struct` should be saved to disk. [savefile](https://github.com/avl/savefile) allows for serialization and compression of a rust data-structure while [directories-rs](https://github.com/dirs-dev/directories-rs) decides where that file should go.

This crate is **not** meant to be used as a database or anything more complex then a simple `struct`. Carrying over from [savefile](https://github.com/avl/savefile), this crate could be used, for example, for a save file in a game.

### Why

I have been making a few toy program's in rust and kept finding a need to have some form of persistent storage. I did not want anything that was complicated to implement, and so, something as simple as attaching an attribute to a struct seemed like a good idea.

## Usage

> [!NOTE]
> Currently, only `structs` have been tested and are the scope of this crate.

### Requirements

- [Rust](https://www.rust-lang.org/) == 2021

### Cargo

```sh
cargo add localsavefile
```

### Minimal Example

> [!IMPORTANT]
> Your struct **must** derive Default, this is used when loading from disk.

```rust
#[localsavefile]
#[derive(Default)]
struct MySave {
    val: u32,
}

let foo = MySave { val: 21 };
foo.save();
let bar = MySave::load_default();
let mut baz = MySave { val: 42 };
baz.load();
assert_eq!(foo.val, bar.val); // Should never trigger
assert_eq!(bar.val, foo.val); // Should never trigger
MySave::remove_file();
```

### Persistent File

If you wish to maintain the underlying file open, as in, not having to reopen it each time `save` or `load` is called, a file handler can be added to your struct through the parameter `persist = true`. This will modify your struct and add an additional field. It's usage is the same as the non-persistent version, with a few caveats as shown.

```rust
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
// foo.open_default(); // You should call open or open_default first
// but foo.save() will also open_default if needed
foo.save(); // Save now requires foo to be mutable
let mut bar = MySavePersist::load_default();
assert_eq!(foo.val, bar.val); // Should never trigger
foo.close();
bar.close(); // Requires bar to be mutable
// Close any instances before removing the file
MySavePersist::remove_file();
```

> [!CAUTION]
> Because `localsavefile(persist = true)` modifies your struct, it is important to place it before any derives, as such.
>
> ```rust
> // First localsavefile
> #[localsavefile(persist = true)]
> // Then whatever else ...
> #[derive(Default)]
> struct MySave {
>     val: u32,
>     // HIDDEN: __place_localsavefile_above_any_derives : Option<File>
> }
> ```
>
> In this case, this ensures the added field gets processed by `Default`.

### Options

By default, the underlying file name is based off a sanitized combination of [`module_path!`](https://doc.rust-lang.org/std/macro.module_path.html), called from where the struct is defined, and the `struct` name.

The directory where files are stored is based off of [`directories::ProjectDirs.data_dir`](https://github.com/dirs-dev/directories-rs?tab=readme-ov-file#projectdirs), where the name and first author in your `Cargo.toml` are used as parameters. Author does not need to be defined, but should be anyways.

As you can imagine, changing anything that these defaults use will sneakily change what your struct loads. The following options shown allow to override any of the mention values to maintain a static path.

```rust
#[localsavefile(name = "a_unique_name", path = "./a/valid/path")]
#[derive(Default)]
struct TestStruct {
    val: u32,
    str: String,
}
```

The following is what the [Minimal Example](#minimal-example) will output as on my windows machine using `localsavefile-test`.

```bash
C:\\Users\\%USER%\\AppData\\Roaming\\localsavefile-test-author🧪\\localsavefile-test\\data\\localsavefile_test.mysave.bin
```

The version option takes a `u32` and is passed to the underlying [savefile](https://github.com/avl/savefile) crate. Take a look at the [version section](https://docs.rs/savefile/latest/savefile/#handling-old-versions) of that crate for more information, as that is all still relevant on this struct.

```rust
#[localsavefile(version = 1)]
#[derive(Default)]
struct TestStruct {
    val: u32,
    #[savefile_default_val = "blank"]
    #[savefile_versions = "0..0"]
    str: String,
}
```

## Related

- avl/[savefile](https://github.com/avl/savefile)
- dirs-dev/[directories-rs](https://github.com/dirs-dev/directories-rs)

## License

Licensed under either of

- Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

See [CONTRIBUTING.md](CONTRIBUTING.md).
