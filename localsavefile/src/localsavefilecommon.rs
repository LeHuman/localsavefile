use std::{
    fs::{self, File},
    io::{self, BufReader, BufWriter},
    path::PathBuf,
};

use savefile::{Deserialize, Serialize};
use tracing::{debug, error};

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

    fn save_file(&self, file_path: &str) -> io::Result<()> {
        let mut writer = BufWriter::new(File::create(file_path)?);
        let result = savefile::save_compressed(&mut writer, Self::get_version(), self);
        match result {
            Err(err) => Err(std::io::Error::other(err)),
            _ => Ok(()),
        }
    }

    fn load_file(&mut self, file_path: &str) -> io::Result<()> {
        let mut reader = BufReader::new(File::open(file_path)?);

        let result: Result<Self, savefile::SavefileError> =
            savefile::load(&mut reader, Self::get_version());
        match result {
            Ok(res) => {
                *self = res;
                Ok(())
            }
            Err(err) => Err(std::io::Error::other(err)),
        }
    }

    // NOTE: Ensure any instances have closed their files for the following functions
    fn remove_file() -> io::Result<()> {
        fs::remove_file(Self::get_full_path()?)?;
        Ok(())
    }

    fn replace_file(file_path: &str) -> io::Result<()> {
        let file_path = std::path::Path::new(file_path);
        if !file_path.is_file() {
            error!(
                "Given path does not exist or is not a file: {:?}",
                file_path
            );
            return Err(std::io::Error::other(
                "Given path does not exist or is not a file",
            ));
        }
        let save_path = Self::get_full_path()?;
        if save_path.exists() {
            if save_path.is_file() {
                fs::remove_file(&save_path)?;
            } else {
                // NOTE: Shouldn't be reached, but just in case
                fs::remove_dir(&save_path)?;
            }
        }
        fs::copy(file_path, save_path)?;
        Ok(())
    }
}
