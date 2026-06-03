use std::sync::Arc;
use std::sync::atomic::Ordering::Relaxed;
use std::{path::Path, sync::atomic::AtomicBool};

use badger_rs::{BadgerError, Database};

use crate::traits::{Db, SnapshotCreator};
use crate::{Iter, ReaderWriterIterType, Snapshot};
use crate::{ReaderWriterIter, Writer, badger::snapshot::BadgerSnapshotIter};

use crate::{Reader, badger::BadgerSnapshot};

#[derive(Debug, thiserror::Error)]
pub enum BadgerDbError {
    #[error("badger error: {0}")]
    BadgerError(#[from] BadgerError),
    #[error("snapshot error: {0}")]
    SnapshotError(#[from] crate::badger::snapshot::BadgerSnapshotError),
    #[error("datastore is closed")]
    Closed,
}

#[derive(Debug, Clone)]
pub struct BadgerDb {
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    handle: Database,
    closed: AtomicBool,
}

#[derive(Debug, Clone, Default)]
pub struct OpenOptions(badger_rs::OpenOptions);

#[derive(Debug, Clone)]
pub struct OpenOptionsBuilder(OpenOptions);

impl OpenOptions {
    pub fn builder() -> OpenOptionsBuilder {
        OpenOptionsBuilder(OpenOptions::default())
    }
}

impl OpenOptionsBuilder {
    pub fn value_dir(mut self, value_dir: impl Into<String>) -> Self {
        self.0.0.value_dir = value_dir.into();
        self
    }

    pub fn in_memory(mut self, in_memory: bool) -> Self {
        self.0.0.in_memory = in_memory;
        self
    }

    pub fn encryption_key(mut self, encryption_key: Vec<u8>) -> Self {
        self.0.0.encryption_key = encryption_key;
        self
    }

    pub fn index_cache_size(mut self, index_cache_size: i64) -> Self {
        self.0.0.index_cache_size = index_cache_size;
        self
    }

    pub fn value_log_file_size(mut self, value_log_file_size: i64) -> Self {
        self.0.0.value_log_file_size = value_log_file_size;
        self
    }

    pub fn build(self) -> OpenOptions {
        OpenOptions(self.0.into())
    }
}

impl From<OpenOptions> for badger_rs::OpenOptions {
    fn from(opts: OpenOptions) -> Self {
        opts.0
    }
}

impl ReaderWriterIterType for BadgerDb {}

impl BadgerDb {
    pub fn new(path: impl AsRef<Path>, options: OpenOptions) -> Result<Self, BadgerDbError> {
        let handle = badger_rs::Database::open(path, &options.into())?;
        Ok(Self {
            inner: Arc::new(Inner {
                handle,
                closed: AtomicBool::new(false),
            }),
        })
    }
}

impl Db for BadgerDb {
    type DbError = BadgerDbError;

    fn close(&self) {
        if !self.inner.closed.load(Relaxed) {
            self.inner.handle.close().ok();
            self.inner.closed.store(true, Relaxed);
        }
    }

    fn drop_all(&self) -> Result<(), Self::DbError> {
        if self.inner.closed.load(Relaxed) {
            return Err(BadgerDbError::Closed);
        }
        self.inner
            .handle
            .drop_all()
            .map_err(BadgerDbError::BadgerError)
    }
}

impl SnapshotCreator for BadgerDb {
    type Snapshot = BadgerSnapshot;
    type Error = BadgerDbError;

    fn create_read_only_snapshot(&self) -> Result<BadgerSnapshot, BadgerDbError> {
        if self.inner.closed.load(Relaxed) {
            return Err(BadgerDbError::Closed);
        }
        self.inner
            .handle
            .new_txn(true)
            .map(|txn| BadgerSnapshot(Arc::new(txn)))
            .map_err(BadgerDbError::BadgerError)
    }

    fn create_read_write_snapshot(&self) -> Result<BadgerSnapshot, BadgerDbError> {
        if self.inner.closed.load(Relaxed) {
            return Err(BadgerDbError::Closed);
        }
        self.inner
            .handle
            .new_txn(false)
            .map(|txn| BadgerSnapshot(Arc::new(txn)))
            .map_err(BadgerDbError::BadgerError)
    }
}

impl Reader for BadgerDb {
    type Error = BadgerDbError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        if self.inner.closed.load(Relaxed) {
            return Err(BadgerDbError::Closed);
        }

        let c_snapshot = self.create_read_only_snapshot()?;
        let res = c_snapshot.get(key).or_else(|e| {
            if let super::snapshot::BadgerSnapshotError::BadgerError(
                badger_rs::BadgerError::NotFound,
            ) = e
            {
                Ok(None)
            } else {
                Err(BadgerDbError::SnapshotError(e))
            }
        });
        c_snapshot.discard();
        res
    }

    fn has(&self, key: &[u8]) -> Result<bool, Self::Error> {
        if self.inner.closed.load(Relaxed) {
            return Err(BadgerDbError::Closed);
        }
        let c_snapshot = self.create_read_only_snapshot()?;
        let res = c_snapshot.has(key).map_err(BadgerDbError::SnapshotError);
        c_snapshot.discard();
        res
    }
}

impl ReaderWriterIter for BadgerDb {
    type IterError = BadgerDbError;
    type Iter = BadgerSnapshotIter;

    fn iter(&self, opts: crate::IterOptions) -> Result<Self::Iter, Self::IterError> {
        if self.inner.closed.load(Relaxed) {
            return Err(BadgerDbError::Closed);
        }
        let c_snapshot = self.create_read_only_snapshot()?;
        c_snapshot
            .0
            .iterator(&opts.clone().into())
            .map(|biter| BadgerSnapshotIter::new(biter, Some(c_snapshot), opts.keys_only()))
            .map_err(BadgerDbError::BadgerError)
    }
}

/// reproduces same behavior as iterator.withCloser via rust ownership
/// ref: /corekv/badger_ffi/badger.go ->
/// ```ignore
/// it.withCloser(func() error {
///     txn.Discard()
///     return nil
/// })
/// ```
impl Drop for BadgerSnapshotIter {
    fn drop(&mut self) {
        if let Some(owned_txn) = self.owned_txn.take() {
            let _ = self.close();
            owned_txn.discard();
        }
    }
}

impl Writer for BadgerDb {
    type Error = BadgerDbError;

    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        if self.inner.closed.load(Relaxed) {
            return Err(BadgerDbError::Closed);
        }
        let mut c_snapshot = self.create_read_write_snapshot()?;
        c_snapshot
            .set(key, value)
            .map_err(BadgerDbError::SnapshotError)?;
        c_snapshot.commit().map_err(BadgerDbError::SnapshotError)?;
        c_snapshot.discard();
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), Self::Error> {
        if self.inner.closed.load(Relaxed) {
            return Err(BadgerDbError::Closed);
        }
        let mut c_snapshot = self.create_read_write_snapshot()?;
        c_snapshot
            .delete(key)
            .map_err(BadgerDbError::SnapshotError)?;
        c_snapshot.commit().map_err(BadgerDbError::SnapshotError)?;
        c_snapshot.discard();
        Ok(())
    }
}

// maybe add txn discard test if db is used with it's iter owned snapshot discrad behaviour