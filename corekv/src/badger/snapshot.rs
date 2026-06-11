use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;

use crate::traits::{ErrorFamily, IterOptions};
use crate::{NewIter, Snapshot};

use crate::{Iter, Reader, Writer};

#[derive(Debug, thiserror::Error)]
pub enum BadgerSnapshotError {
    #[error("snapshot is discarded")]
    Discarded,

    #[error("snapshot get failed")]
    Get(#[source] badger_rs::BadgerError),

    #[error("snapshot has failed")]
    Has(#[source] badger_rs::BadgerError),

    #[error("snapshot set failed")]
    Set(#[source] badger_rs::BadgerError),

    #[error("snapshot delete failed")]
    Delete(#[source] badger_rs::BadgerError),

    #[error("snapshot iter creation failed")]
    IterCreate(#[source] badger_rs::BadgerError),

    #[error("snapshot commit failed")]
    Commit(#[source] badger_rs::BadgerError),
}

#[derive(Debug, thiserror::Error)]
pub enum BadgerSnapshotIterError {
    #[error("snapshot iter has no selected entry")]
    NoEntrySelected,

    #[error("snapshot iter is keys-only")]
    KeysOnly,

    #[error("snapshot iter is closed")]
    Closed,

    #[error("snapshot iter next failed")]
    Next(#[source] badger_rs::BadgerError),

    #[error("snapshot iter key failed")]
    Key(#[source] badger_rs::BadgerError),

    #[error("snapshot iter value failed")]
    Value(#[source] badger_rs::BadgerError),

    #[error("snapshot iter seek failed")]
    Seek(#[source] badger_rs::BadgerError),

    #[error("snapshot iter reset failed")]
    Reset(#[source] badger_rs::BadgerError),

    #[error("snapshot iter close failed")]
    Close(#[source] badger_rs::BadgerError),
}

impl ErrorFamily for BadgerSnapshot {
    type AccessError = BadgerSnapshotError;
    type IterError = BadgerSnapshotIterError;
    type SnapshotError = BadgerSnapshotError;
}

#[derive(Debug)]
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

impl Snapshot for BadgerSnapshot {
    fn commit(&self) -> Result<(), Self::AccessError> {
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0.commit().map_err(BadgerSnapshotError::Commit)
    }

    fn discard(&self) {
        if self.0.is_discarded() {
            return;
        }
        self.0.discard()
    }
}

impl Reader for BadgerSnapshot {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::AccessError> {
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0.get(key).map(|v| Some(v.to_vec())).or_else(|e| {
            if let badger_rs::BadgerError::NotFound = e {
                Ok(None)
            } else {
                Err(BadgerSnapshotError::Get(e))
            }
        })
    }

    fn has(&self, key: &[u8]) -> Result<bool, Self::AccessError> {
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0.has(key).map_err(BadgerSnapshotError::Has)
    }
}

impl NewIter for BadgerSnapshot {
    type Iter = BadgerSnapshotIter;

    fn iter(&self, opts: IterOptions) -> Result<Self::Iter, Self::AccessError> {
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
            .map_err(BadgerSnapshotError::IterCreate)
    }
}

impl Writer for BadgerSnapshot {
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::AccessError> {
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0.set(key, value).map_err(BadgerSnapshotError::Set)
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), Self::AccessError> {
        if self.0.is_discarded() {
            return Err(BadgerSnapshotError::Discarded);
        }
        self.0.delete(key).map_err(BadgerSnapshotError::Delete)
    }
}

impl Iter for BadgerSnapshotIter {
    type IterError = BadgerSnapshotIterError;

    fn next(&mut self) -> Result<bool, Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::Closed);
        }
        let res = self
            .inner
            .has_next()
            .map_err(BadgerSnapshotIterError::Next)?;

        if !self.selected_entry.load(Relaxed) && res {
            self.selected_entry.store(true, Relaxed);
        }
        Ok(res)
    }

    fn key(&self) -> Result<Vec<u8>, Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::Closed);
        }
        if !self.selected_entry.load(Relaxed) {
            return Err(BadgerSnapshotIterError::NoEntrySelected);
        }
        self.inner.key().map_err(BadgerSnapshotIterError::Key)
    }

    fn value(&self) -> Result<Vec<u8>, Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::Closed);
        }
        if self.keys_only {
            return Err(BadgerSnapshotIterError::KeysOnly);
        }
        if !self.selected_entry.load(Relaxed) {
            return Err(BadgerSnapshotIterError::NoEntrySelected);
        }
        self.inner.value().map_err(BadgerSnapshotIterError::Value)
    }

    fn seek(&mut self, key: &[u8]) -> Result<bool, Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::Closed);
        }
        let res = self
            .inner
            .seek(key)
            .map_err(BadgerSnapshotIterError::Seek)?;
        if res {
            self.selected_entry.store(true, Relaxed);
        }
        Ok(res)
    }

    fn reset(&mut self) -> Result<(), Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::Closed);
        }
        self.inner.reset().map_err(BadgerSnapshotIterError::Reset)?;
        self.selected_entry.store(false, Relaxed);
        Ok(())
    }

    fn close(&mut self) -> Result<(), Self::IterError> {
        if self.inner.is_closed() {
            return Err(BadgerSnapshotIterError::Closed);
        }
        self.inner.close().map_err(BadgerSnapshotIterError::Close)
    }
}
