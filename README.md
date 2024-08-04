<!-- PROJECT: localsavefile -->
<!-- TITLE: localsavefile -->
<!-- KEYWORDS: library  -->
<!-- LANGUAGES: Rust -->
<!-- STATUS: Semi-Active -->

# LocalSaveFile

[![Crates.io](https://img.shields.io/crates/v/localsavefile.svg)](https://crates.io/crates/localsavefile)
[![Docs.rs](https://docs.rs/localsavefile/badge.svg)](https://docs.rs/localsavefile)
[![CI](https://github.com/lehuman/localsavefile/workflows/CI/badge.svg)](https://github.com/lehuman/localsavefile/actions)

[About](#about) - [Usage](#usage) - [Related](#related) - [License](#license) - [Contribution](#contribution)

## Status

**`Semi-Active`**

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
- [savefile](https://github.com/avl/savefile) >= 0.17.6

### Cargo

```sh
cargo add localsavefile savefile
```

> [!IMPORTANT]
> As this is mainly a convenience wrapper, [savefile](https://github.com/avl/savefile) also needs to be added with cargo to be used by the exported macros.

### Minimal Example

> [!NOTE]
> The macros Default and Savefile are automatically set to be derived. In any case, use `localsavefile_impl` instead of `localsavefile` to manually derive them.

```rust
use localsavefile::{localsavefile, LocalSaveFile, LocalSaveFileCommon};

#[localsavefile]
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

> [!WARNING]
> If, for whatever reason, you implement `localsavefile` in a library, it is recommened to re-export the macro `setlsf` and have the user call this macro before anything. It will set the env variables `LOCAL_SAVE_FILE_CARGO_PKG_NAME` and `LOCAL_SAVE_FILE_CARGO_PKG_AUTHORS` to be used in place of `CARGO_PKG_NAME` and `CARGO_PKG_AUTHORS` respectively. Otherwise, the default paths will be in regards to your crate, not the user's.
>
> ```rust
> // In lib.rs, probably
> pub use localsavefile::setlsf;
> ```

### Persistent File

If you wish to maintain the underlying file open, as in, not having to reopen it each time `save` or `load` is called, a file handler can be added to your struct through the parameter `persist = true`. This will modify your struct and add an additional field. It's usage is the same as the non-persistent version, with a few caveats as shown.

> [!NOTE]
> Persistent localsavefiles will store it's path upon loading or saving. This means any subsequent calls to `setlsf` will not affect it.

```rust
use localsavefile::{localsavefile, LocalSaveFilePersistent, LocalSaveFileCommon};

#[localsavefile(persist = true)]
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
> Because `localsavefile(persist = true)` modifies your struct, it is important to place it before any derives that must be aware of every field, such as when using `localsavefile_impl`.
>
> ```rust
> // First localsavefile
> #[localsavefile_impl(persist = true)]
> // Then whatever else ...
> #[derive(Savefile, Default)]
> struct MySave {
>     val: u32,
>     // HIDDEN: __place_localsavefile_above_any_derives : Option<File>
> }
> ```
>
> In this case, this ensures the added field gets processed by `Default` and `Savefile`.

### Options

By default, the underlying file name is based off a sanitized combination of [`module_path!`](https://doc.rust-lang.org/std/macro.module_path.html), called from where the struct is defined, and the `struct` name.

The directory where files are stored is based off of [`directories::ProjectDirs.data_dir`](https://github.com/dirs-dev/directories-rs?tab=readme-ov-file#projectdirs), where the name and first author in your `Cargo.toml` are used as parameters. Author does not need to be defined, but should be anyways.

As you can imagine, changing anything that these defaults use will sneakily change what your struct loads. The following options shown allow to override any of the mention values to maintain a static path.

```rust
#[localsavefile(name = "a_unique_name", path = "./a/valid/path")]
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
struct TestStruct {
    val: u32,
    #[savefile_default_val = "not-blank"]
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
