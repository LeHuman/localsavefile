use savefile::prelude::SavefileNoIntrospect;
use std::fs::File;

mod localsavefilecommon;

mod localsavefile;
mod localsavefilepersistent;

pub use crate::localsavefile::LocalSaveFile;
pub use crate::localsavefilecommon::LocalSaveFileCommon;
pub use crate::localsavefilepersistent::LocalSaveFilePersistent;
pub use localsavefile_derive::localsavefile;
pub use sanitize_filename;

#[derive(Default, SavefileNoIntrospect, Debug)]
pub struct LocalSaveFileMetaData {
    #[savefile_ignore]
    pub file: Option<File>,
}
impl PartialEq for LocalSaveFileMetaData {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl PartialOrd for LocalSaveFileMetaData {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}
impl Clone for LocalSaveFileMetaData {
    fn clone(&self) -> Self {
        Self {
            file: self.file.as_ref().and_then(|file| file.try_clone().ok()),
        }
    }
}
