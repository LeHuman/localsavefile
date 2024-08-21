use std::{
    fs::File,
    io::{self, BufReader, BufWriter},
};

use tracing::{debug, warn};

use crate::LocalSaveFileCommon;

pub trait LocalSaveFile
where
    Self: LocalSaveFileCommon,
{
    fn save(&self) -> io::Result<()> {
        let mut writer = BufWriter::new(File::create(Self::get_full_path()?)?);
        let result = savefile::save_compressed(&mut writer, Self::get_version(), self);
        match result {
            Err(err) => Err(std::io::Error::other(err)),
            _ => Ok(()),
        }
    }

    fn load(&mut self) -> io::Result<()> {
        let mut reader = BufReader::new(File::open(Self::get_full_path()?)?);

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

    fn load_default() -> Self {
        let mut def = Self::default();
        let result = def.load();
        if result.is_err() {
            debug!("{:?}", result);
            warn!(
                "Failed to load on LocalSaveFile default, using default {:?}",
                Self::get_struct_name()
            );
        }
        def
    }

    fn load_file_or_default(file_path: &str) -> Self {
        let mut def = Self::default();
        let result = def.load_file(file_path);
        if result.is_err() {
            debug!("{:?}", result);
            warn!(
                "Failed to load on LocalSaveFile, using path {:?}",
                file_path
            );
        }
        def
    }
}
