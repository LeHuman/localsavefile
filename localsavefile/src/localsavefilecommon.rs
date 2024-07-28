use std::{fs, io, path::PathBuf};

use savefile::{Deserialize, Serialize};
use tracing::debug;

pub trait LocalSaveFileCommon
where
    Self: Serialize + Deserialize + Default,
{
    fn get_struct_name() -> String;
    fn get_version() -> u32;
    fn get_pkg_name() -> String;
    fn get_pkg_author() -> String;

    fn get_dir_path() -> io::Result<PathBuf> {
        let pkg_name = Self::get_pkg_name();
        let pkg_authors = Self::get_pkg_author(); // IMPROVE: Better author string
        let dir = directories::ProjectDirs::from(
            "app.rs", // TODO: option to change this qualifier?
            if pkg_authors.is_empty() {
                "Someone"
            } else {
                &pkg_authors
            },
            if pkg_name.is_empty() {
                "SomeRustApp"
            } else {
                &pkg_name
            },
        );

        let Some(dir) = dir else {
            return Err(std::io::Error::other("Failed to get file directory"));
        };

        Ok(PathBuf::from(dir.data_dir()))
    }

    fn get_full_path() -> io::Result<PathBuf> {
        let path = Self::get_dir_path()?;
        fs::create_dir_all(&path)?;
        let path = path.join(Self::get_struct_name() + ".bin");
        debug!("Default Full Path {:?}", path.to_str().unwrap_or("FAILED"));
        Ok(path)
    }

    // NOTE: Ensure any instances have closed their files
    fn remove_file() -> io::Result<()> {
        fs::remove_file(Self::get_full_path()?)?;
        Ok(())
    }
}
