use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, BufWriter, Seek, Write},
    path::{Path, PathBuf},
};

use savefile::prelude::SavefileNoIntrospect;
use tracing::{debug, warn};

use crate::LocalSaveFileCommon;

#[derive(Default, SavefileNoIntrospect, Debug)]
pub struct LocalSaveFileMetaData {
    #[savefile_ignore]
    pub file: Option<File>,
    #[savefile_ignore]
    pub reader: Option<BufReader<File>>,
    #[savefile_ignore]
    pub path: Option<PathBuf>,
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
    #[allow(clippy::non_canonical_partial_ord_impl)]
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
            reader: None,
            path: None,
        }
    }
}

pub trait LocalSaveFilePersistent
where
    Self: LocalSaveFileCommon,
{
    fn get_metadata_mut(&mut self) -> &mut LocalSaveFileMetaData;

    fn close(&mut self) {
        if self.get_metadata_mut().file.is_none() {
            debug!("LocalSaveFile already closed");
            return;
        }
        let metadata = self.get_metadata_mut();
        metadata.file = None;
        metadata.reader = None;
    }

    // NOTE: Does not ensure path is valid/existing
    fn open<P>(&mut self, path: P) -> io::Result<()>
    where
        P: AsRef<Path>,
    {
        let metadata = self.get_metadata_mut();
        let file = match &mut metadata.file {
            Some(file) => {
                debug!("LocalSaveFile already open with file");
                file
            }
            None => {
                metadata.file = Some(
                    OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .truncate(false)
                        .open(path)?,
                );
                metadata.reader = None;
                metadata.file.as_mut().expect("Failed to get metadata file")
            }
        };

        if metadata.reader.is_none() {
            let reader = BufReader::new(file.try_clone()?);
            metadata.reader = Some(reader);
        }

        Ok(())
    }

    fn save(&mut self) -> io::Result<()> {
        self.open_default()?;
        let metadata = self.get_metadata_mut();
        // Clear file using existing handle
        let Some(file) = &mut metadata.file else {
            return Err(std::io::Error::other("Failed to get file handle"));
        };
        file.flush()?;
        file.rewind()?;
        file.set_len(0)?;

        // Clone handle into writer
        let mut file = file.try_clone()?;
        let mut writer = BufWriter::new(&mut file);

        let result = savefile::save_compressed(&mut writer, Self::get_version(), self);
        if let Err(err) = result {
            Err(std::io::Error::other(err))
        } else {
            Ok(())
        }
    }

    fn load(&mut self) -> io::Result<()> {
        self.open_default()?;

        let metadata = self.get_metadata_mut();
        let Some(file) = metadata.file.as_mut() else {
            return Err(io::Error::other("Failed to get file handle"));
        };
        file.flush()?;
        file.rewind()?;

        let Some(mut reader) = metadata.reader.as_mut() else {
            return Err(std::io::Error::other("Failed to get reader"));
        };

        let result: Result<Self, savefile::SavefileError> =
            savefile::load(&mut reader, Self::get_version());
        match result {
            Ok(res) => {
                let metadata = metadata.clone();
                *self = res;
                *self.get_metadata_mut() = metadata;
                Ok(())
            }
            Err(err) => Err(std::io::Error::other(err)),
        }
    }

    fn open_default(&mut self) -> io::Result<()> {
        let path = match &self.get_metadata_mut().path {
            Some(p) => p.to_owned(),
            None => Self::get_full_path()?,
        };
        self.open(path)
    }

    fn load_default() -> Self {
        let mut def = Self::default();
        if def.open_default().is_err() {
            warn!("Failed to open default LocalSaveFile directory");
        }
        if let Err(result) = def.load() {
            debug!("{:?}", result);
            warn!(
                "Failed to load on LocalSaveFile default, using default {:?}",
                Self::get_struct_name()
            );
        }
        def
    }
}
