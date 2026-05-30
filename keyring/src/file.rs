use std::{fs::DirBuilder, os::unix::fs::DirBuilderExt as _, path::PathBuf};

use thiserror::Error;

use crate::KeyringT;

// this error derive IoError
#[derive(Error, Debug)]
pub enum KeyringFileError {
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug)]
pub struct KeyringFile<'a> {
    dir: PathBuf,
    password: &'a [u8],
}

impl<'a> KeyringFile<'a> {
    pub fn open(dir: impl Into<PathBuf>, password: &'a [u8]) -> Result<Self, KeyringFileError> {
        let dir = dir.into();
        DirBuilder::new()
            .recursive(true)
            .mode(0o755)
            .create(&dir)?;
        Ok(Self {
            dir,
            password,
        })
    }
}

impl KeyringT for KeyringFile<'_> {
    type Error = KeyringFileError;

    fn set(&mut self, name: &str, key: &[u8]) -> Result<(), Self::Error> {
       
        todo!("")
    }

    fn get(&self, name: &str) -> Result<Vec<u8>, Self::Error> {
        todo!()
    }

    fn delete(&mut self, name: &str) -> Result<(), Self::Error> {
        todo!()
    }

    fn list(&self) -> Result<Vec<String>, Self::Error> {
        todo!()
    }
}
