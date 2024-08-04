mod localsavefilecommon;

mod localsavefile;
mod localsavefilepersistent;

pub use crate::localsavefile::LocalSaveFile;
pub use crate::localsavefilecommon::LocalSaveFileCommon;
pub use crate::localsavefilepersistent::LocalSaveFileMetaData;
pub use crate::localsavefilepersistent::LocalSaveFilePersistent;
pub use localsavefile_derive::localsavefile;
pub use localsavefile_derive::localsavefile_impl;
pub use sanitize_filename::sanitize;

#[macro_export]
macro_rules! setlsf {
    () => {
        std::env::set_var("LOCAL_SAVE_FILE_CARGO_PKG_NAME", env!("CARGO_PKG_NAME"));
        std::env::set_var(
            "LOCAL_SAVE_FILE_CARGO_PKG_AUTHORS",
            env!("CARGO_PKG_AUTHORS"),
        );
    };
}
