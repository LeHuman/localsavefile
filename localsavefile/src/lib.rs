use savefile::prelude::SavefileNoIntrospect;
use std::fs::File;

mod localsavefilecommon;

mod localsavefile;
mod localsavefilepersistent;

pub use crate::localsavefile::LocalSaveFile;
pub use crate::localsavefilecommon::LocalSaveFileCommon;
pub use crate::localsavefilepersistent::LocalSaveFilePersistent;
pub use localsavefile_derive::localsavefile;
pub use localsavefile_derive::localsavefile_impl;
pub use sanitize_filename::sanitize;

#[derive(Default, SavefileNoIntrospect, Debug)]
pub struct LocalSaveFileMetaData {
    #[savefile_ignore]
    pub file: Option<File>,
}
impl core::hash::Hash for LocalSaveFileMetaData {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {}
}
impl PartialEq for LocalSaveFileMetaData {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl Eq for LocalSaveFileMetaData {}
impl PartialOrd for LocalSaveFileMetaData {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        Some(std::cmp::Ordering::Equal)
    }
}
impl Ord for LocalSaveFileMetaData {
    fn cmp(&self, _other: &Self) -> std::cmp::Ordering {
        std::cmp::Ordering::Equal
    }
}
impl Clone for LocalSaveFileMetaData {
    fn clone(&self) -> Self {
        Self {
            file: self.file.as_ref().and_then(|file| file.try_clone().ok()),
        }
    }
}
