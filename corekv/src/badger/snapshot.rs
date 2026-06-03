use std::sync::Arc;
use std::sync::atomic::AtomicBool;

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
    #[error(
        "no entry selected: call next() or seek() to select an entry before calling key() or value()"
    )]
    NoEntrySelected,
    #[error(
        "keys only iterator: value() is not supported on an iterator created with keys_only=true"
    )]
    KeysOnly,
}

pub struct BadgerSnapshotIter {
    inner: badger_rs::BadgerIterator,
    pub(crate) owned_txn: Option<BadgerSnapshot>,
    selected_entry: AtomicBool,
    keys_only: bool,
}

impl BadgerSnapshotIter {
    pub fn new(
        inner: badger_rs::BadgerIterator,
        owned_txn: Option<BadgerSnapshot>,
        keys_only: bool,
    ) -> Self {
        Self {
            inner,
            owned_txn,
            selected_entry: AtomicBool::new(false),
            keys_only,
        }
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
            .iterator(&opts.clone().into())
            .map(|biter| BadgerSnapshotIter {
                inner: biter,
                owned_txn: None,
                selected_entry: AtomicBool::new(false),
                keys_only: opts.keys_only(),
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

    fn next(&mut self) -> Result<bool, Self::IterError> {
        let res = self
            .inner
            .has_next()
            .map_err(BadgerSnapshotIterError::BadgerError)?;
        if res {
            self.selected_entry
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(res)
    }

    fn key(&self) -> Result<Vec<u8>, Self::IterError> {
        if !self
            .selected_entry
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(BadgerSnapshotIterError::NoEntrySelected);
        }
        self.inner
            .key()
            .map_err(BadgerSnapshotIterError::BadgerError)
    }

    fn value(&self) -> Result<Vec<u8>, Self::IterError> {
        if self.keys_only {
            return Err(BadgerSnapshotIterError::KeysOnly);
        }
        if !self
            .selected_entry
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(BadgerSnapshotIterError::NoEntrySelected);
        }
        self.inner
            .value()
            .map_err(BadgerSnapshotIterError::BadgerError)
    }

    fn seek(&mut self, key: &[u8]) -> Result<bool, Self::IterError> {
        let res = self
            .inner
            .seek(key)
            .map_err(BadgerSnapshotIterError::BadgerError)?;
        if res {
            self.selected_entry
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(res)
    }

    fn reset(&mut self) -> Result<(), Self::IterError> {
        self.inner
            .reset()
            .map_err(BadgerSnapshotIterError::BadgerError)?;
        self.selected_entry
            .store(false, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    fn close(&mut self) -> Result<(), Self::IterError> {
        self.inner
            .close()
            .map_err(BadgerSnapshotIterError::BadgerError)
    }
}