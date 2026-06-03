use std::sync::Arc;

use crate::traits::IterOptions;
use crate::{ReaderWriterIter, ReaderWriterIterType, Snapshot};

use crate::{Iter, Reader, Writer};

#[derive(Debug, thiserror::Error)]
pub enum BadgerSnapshotError {
    #[error("badger error: {0}")]
    BadgerError(#[from] badger_rs::BadgerError),
}

#[derive(Debug, thiserror::Error)]
pub enum BadgerSnapshotIterError {
    #[error("badger error: {0}")]
    BadgerError(#[from] badger_rs::BadgerError),
}

pub struct BadgerSnapshotIter {
    inner: badger_rs::BadgerIterator,
    pub(crate) owned_txn: Option<BadgerSnapshot>,
}

impl BadgerSnapshotIter {
    pub fn new(inner: badger_rs::BadgerIterator, owned_txn: Option<BadgerSnapshot>) -> Self {
        Self { inner, owned_txn }
    }
}

#[derive(Debug, Clone)]
pub struct BadgerSnapshot(pub Arc<badger_rs::Txn>);

impl ReaderWriterIterType for BadgerSnapshot {}

impl Snapshot for BadgerSnapshot {
    type SnapshotError = BadgerSnapshotError;

    fn commit(&self) -> Result<(), BadgerSnapshotError> {
        self.0.commit().map_err(BadgerSnapshotError::BadgerError)
    }

    fn discard(&self) {
        self.0.discard()
    }
}

impl Reader for BadgerSnapshot {
    type Error = BadgerSnapshotError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        self.0.get(key).map(|v| Some(v.to_vec())).or_else(|e| {
            if let badger_rs::BadgerError::NotFound = e {
                Ok(None)
            } else {
                Err(BadgerSnapshotError::BadgerError(e))
            }
        })
    }

    fn has(&self, key: &[u8]) -> Result<bool, Self::Error> {
        self.0.has(key).map_err(BadgerSnapshotError::BadgerError)
    }
}

impl ReaderWriterIter for BadgerSnapshot {
    type IterError = BadgerSnapshotError;
    type Iter = BadgerSnapshotIter;

    fn iter(&self, opts: IterOptions) -> Result<Self::Iter, Self::IterError> {
        self.0
            .iterator(&opts.into())
            .map(|biter| BadgerSnapshotIter {
                inner: biter,
                owned_txn: None,
            })
            .map_err(BadgerSnapshotError::BadgerError)
    }
}

impl Writer for BadgerSnapshot {
    type Error = BadgerSnapshotError;

    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        self.0
            .set(key, value)
            .map_err(BadgerSnapshotError::BadgerError)
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), Self::Error> {
        self.0.delete(key).map_err(BadgerSnapshotError::BadgerError)
    }
}

impl Iter for BadgerSnapshotIter {
    type IterError = BadgerSnapshotIterError;

    fn has_next(&mut self) -> Result<bool, Self::IterError> {
        self.inner
            .has_next()
            .map_err(BadgerSnapshotIterError::BadgerError)
    }

    fn key(&self) -> Result<Vec<u8>, Self::IterError> {
        self.inner
            .key()
            .map_err(BadgerSnapshotIterError::BadgerError)
    }

    fn value(&self) -> Result<Vec<u8>, Self::IterError> {
        self.inner
            .value()
            .map_err(BadgerSnapshotIterError::BadgerError)
    }
    
    fn seek(&mut self, key: &[u8]) -> Result<bool, Self::IterError> {
        self.inner.seek(key).map_err(BadgerSnapshotIterError::BadgerError)
    }
    
    fn reset(&mut self) -> Result<(), Self::IterError> {
        self.inner.reset().map_err(BadgerSnapshotIterError::BadgerError)
    }
    
    fn close(&mut self) -> Result<(), Self::IterError> {
        self.inner.close().map_err(BadgerSnapshotIterError::BadgerError)
    }
    
}
