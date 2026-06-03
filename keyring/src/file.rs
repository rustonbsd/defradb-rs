use std::{fs::DirBuilder, path::{Path, PathBuf}};

#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt as _;

use thiserror::Error;

use crate::{KeyringT, crypto};

#[derive(Error, Debug)]
pub enum KeyringFileError {
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CryptoError: {0}")]
    CryptoError(#[from] crypto::CryptoError),
}

#[derive(Debug)]
pub struct KeyringFile<'a> {
    dir: PathBuf,
    password: &'a [u8],
}

impl<'a> KeyringFile<'a> {
    pub fn open(dir: impl AsRef<Path>, password: &'a [u8]) -> Result<Self, KeyringFileError> {
        let mut builder = DirBuilder::new();
        builder.recursive(true);

        #[cfg(unix)]
        builder.mode(0o755);

        builder.create(&dir)?;
        Ok(Self { dir: dir.as_ref().to_path_buf(), password })
    }
}

impl KeyringT for KeyringFile<'_> {
    type Error = KeyringFileError;

    fn set(&mut self, name: &str, key: &[u8]) -> Result<(), Self::Error> {
        let enc = crypto::encrypt(key, self.password)?;

        let path = self.dir.join(name);
        std::fs::write(&path, enc)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&path, perms)?;
        }
        Ok(())
    }

    fn get(&self, name: &str) -> Result<Vec<u8>, Self::Error> {
        let path = self.dir.join(name);
        let enc = std::fs::read(&path)?;
        crypto::decrypt(&enc, self.password).map_err(KeyringFileError::CryptoError)
    }

    fn delete(&mut self, name: &str) -> Result<(), Self::Error> {
        let path = self.dir.join(name);
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn list(&self) -> Result<Vec<String>, Self::Error> {
        let files = std::fs::read_dir(&self.dir)?;

        let mut key_names = Vec::new();
        for entry in files {
            let entry = entry?;
            if entry.file_type()?.is_file()
                && let Some(name) = entry.file_name().to_str()
            {
                key_names.push(name.to_string());
            }
        }

        Ok(key_names)
    }
}
