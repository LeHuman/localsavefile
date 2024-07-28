use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Seek, Write},
    path::Path,
};

use tracing::{debug, warn};

use crate::LocalSaveFileCommon;

pub trait LocalSaveFilePersistent
where
    Self: LocalSaveFileCommon,
{
    fn get_file_handle_mut(&mut self) -> &mut Option<File>;

    fn close(&mut self) {
        if self.get_file_handle_mut().is_none() {
            debug!("LocalSaveFile already closed");
            return;
        }
        *self.get_file_handle_mut() = None;
    }

    // NOTE: Does not ensure path is valid/existing
    fn open<P>(&mut self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        if self.get_file_handle_mut().is_some() {
            debug!("LocalSaveFile already open with file");
            return Ok(());
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        *self.get_file_handle_mut() = Some(file);
        Ok(())
    }

    fn save(&mut self) -> io::Result<()> {
        if self.get_file_handle_mut().is_none() {
            warn!("LocalSaveFile does not have file open, using default");
            self.open_default()?;
        }
        // Clear file using existing handle
        let file = self.get_file_handle_mut();
        let Some(file) = file else {
            return Err(std::io::Error::other("Failed to get file handle"));
        };
        file.flush()?;
        file.rewind()?;
        file.set_len(0)?;

        // Clone handle into writer
        let mut file = file.try_clone()?;
        let mut writer = BufWriter::new(&mut file);

        let result = savefile::save_compressed(&mut writer, Self::get_version(), self);
        match result {
            Err(err) => Err(std::io::Error::other(err)),
            _ => Ok(()),
        }
    }

    fn load(&mut self) -> io::Result<()> {
        if self.get_file_handle_mut().is_none() {
            warn!("LocalSaveFile does not have file open, using default");
            self.open_default()?;
        }
        // Get existing handle and rewind to the start of it
        let file = self.get_file_handle_mut();
        let Some(file) = file else {
            return Err(std::io::Error::other("Failed to get file handle"));
        };
        file.rewind()?;

        // Create reader from cloned file handle
        let mut file = file.try_clone()?;
        let mut reader = BufReader::new(&mut file);

        let result: Result<Self, savefile::SavefileError> =
            savefile::load(&mut reader, Self::get_version());
        match result {
            Ok(res) => {
                *self = res;
                *self.get_file_handle_mut() = Some(file);
                Ok(())
            }
            Err(err) => Err(std::io::Error::other(err)),
        }
    }

    fn open_default(&mut self) -> io::Result<()> {
        self.open(Self::get_full_path()?)
    }

    fn load_default() -> Self {
        let mut def = Self::default();
        if def.open_default().is_err() {
            warn!("Failed to open default LocalSaveFile directory");
        }
        let result = def.load();
        if result.is_err() {
            debug!("{:?}", result);
            warn!(
                "Failed to load on LocalSaveFile default, using default {:?}",
                Self::get_struct_name()
            );
        }
        // result.unwrap_or_default()
        def
    }
}
