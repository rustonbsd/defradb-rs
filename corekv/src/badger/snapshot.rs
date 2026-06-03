use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

use crate::traits::IterOptions;
use crate::{ReaderWriterIter, ReaderWriterIterType, Snapshot};

use crate::{Iter, Reader, Writer};

#[derive(Debug, thiserror::Error)]
pub enum BadgerSnapshotError {
    #[error("snapshot discarded: no further operations are allowed")]
    Discarded,
    #[error("badger error: {0}")]
    BadgerError(#[from] badger_rs::BadgerError),
}

#[derive(Debug, thiserror::Error)]
pub enum BadgerSnapshotIterError {
    #[error(
        "no entry selected: call next() or seek() to select an entry before calling key() or value()"
    )]
    NoEntrySelected,
    #[error(
        "keys only iterator: value() is not supported on an iterator created with keys_only=true"
    )]
    KeysOnly,
    #[error("iter is closed: no further operations are allowed")]
    IterClosed,
    #[error("badger error: {0}")]
    BadgerError(#[from] badger_rs::BadgerError),
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
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0.commit().map_err(BadgerSnapshotError::BadgerError)
    }

    fn discard(&self) {
        if self.0.is_discarded() {
            return;
        }
        self.0.discard()
    }
}

impl Reader for BadgerSnapshot {
    type Error = BadgerSnapshotError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0.get(key).map(|v| Some(v.to_vec())).or_else(|e| {
            if let badger_rs::BadgerError::NotFound = e {
                Ok(None)
            } else {
                Err(BadgerSnapshotError::BadgerError(e))
            }
        })
    }

    fn has(&self, key: &[u8]) -> Result<bool, Self::Error> {
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0.has(key).map_err(BadgerSnapshotError::BadgerError)
    }
}

impl ReaderWriterIter for BadgerSnapshot {
    type IterError = BadgerSnapshotError;
    type Iter = BadgerSnapshotIter;

    fn iter(&self, opts: IterOptions) -> Result<Self::Iter, Self::IterError> {
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
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
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0
            .set(key, value)
            .map_err(BadgerSnapshotError::BadgerError)
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), Self::Error> {
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0.delete(key).map_err(BadgerSnapshotError::BadgerError)
    }
}

impl Iter for BadgerSnapshotIter {
    type IterError = BadgerSnapshotIterError;

    fn next(&mut self) -> Result<bool, Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::IterClosed);
        }
        let res = self
            .inner
            .has_next()
            .map_err(BadgerSnapshotIterError::BadgerError)?;
        if !self.selected_entry.load(Relaxed) && res{
            self.selected_entry
                .store(true, Relaxed);
        }
        Ok(res)
    }

    fn key(&self) -> Result<Vec<u8>, Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::IterClosed);
        }
        if !self
            .selected_entry
            .load(Relaxed)
        {
            return Err(BadgerSnapshotIterError::NoEntrySelected);
        }
        self.inner
            .key()
            .map_err(BadgerSnapshotIterError::BadgerError)
    }

    fn value(&self) -> Result<Vec<u8>, Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::IterClosed);
        }
        if self.keys_only {
            return Err(BadgerSnapshotIterError::KeysOnly);
        }
        if !self
            .selected_entry
            .load(Relaxed)
        {
            return Err(BadgerSnapshotIterError::NoEntrySelected);
        }
        self.inner
            .value()
            .map_err(BadgerSnapshotIterError::BadgerError)
    }

    fn seek(&mut self, key: &[u8]) -> Result<bool, Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::IterClosed);
        }
        let res = self
            .inner
            .seek(key)
            .map_err(BadgerSnapshotIterError::BadgerError)?;
        if res {
            self.selected_entry
                .store(true, Relaxed);
        }
        Ok(res)
    }

    fn reset(&mut self) -> Result<(), Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::IterClosed);
        }
        self.inner
            .reset()
            .map_err(BadgerSnapshotIterError::BadgerError)?;
        self.selected_entry
            .store(false, Relaxed);
        Ok(())
    }

    fn close(&mut self) -> Result<(), Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::IterClosed);
        }
        self.inner
            .close()
            .map_err(BadgerSnapshotIterError::BadgerError)
    }
}