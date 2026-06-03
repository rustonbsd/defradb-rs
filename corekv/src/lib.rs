mod badger;
mod namespace;
mod traits;

pub use badger::{BadgerDb, BadgerDbError, OpenOptions};
pub use badger::{
    BadgerSnapshot, BadgerSnapshotError, BadgerSnapshotIter, BadgerSnapshotIterError,
};
pub use traits::{Iter, Reader, ReaderWriterIterType, ReaderWriterIter, Writer, Snapshot, SnapshotCreator, Db, IterOptions, IterOptionsBuilder};

#[derive(Debug, thiserror::Error)]
pub enum CoreKvError {
    #[error("badger error: {0}")]
    BadgerDbError(#[from] BadgerDbError),
    #[error("snapshot error: {0}")]
    SnapshotError(#[from] BadgerSnapshotError),
    #[error("snapshot iterator error: {0}")]
    SnapshotIterError(#[from] BadgerSnapshotIterError),
}