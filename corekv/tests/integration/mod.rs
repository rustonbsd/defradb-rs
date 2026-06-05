mod iterator;
mod close;
mod delete_close;
mod drop_all;
mod get;
mod get_close;
mod has;
mod has_close;
mod set_close;
mod set_delete_get;
mod set_drop_all_has;
mod set_get;
mod set_has;

use std::error::Error;

use corekv::{
    BadgerDb, Chunk, Db, NewIter, OpenOptions, PrefixKey, Reader, Snapshot, SnapshotCreator, Writer,
};

trait DbTest: Db + Reader + Writer + NewIter + SnapshotCreator + Clone
where
    Self::Snapshot: Snapshot + Reader + Writer + NewIter + Clone,
{
}

impl<T> DbTest for T where T: Db + Reader + Writer + NewIter + SnapshotCreator + Clone {}

fn get_base_error(err: &dyn Error) -> &dyn Error {
    let mut current = err;

    while let Some(source) = current.source() {
        current = source;
    }

    current
}

fn open_badger_memory() -> anyhow::Result<BadgerDb> {
    let db = BadgerDb::open("", OpenOptions::builder().in_memory(true).build())?;
    Ok(db)
}

fn open_namespace() -> anyhow::Result<PrefixKey<BadgerDb>> {
    Ok(PrefixKey::wrap(open_badger_memory()?, b"/example".to_vec()))
}

fn open_chunk() -> anyhow::Result<Chunk<BadgerDb>> {
    Ok(Chunk::wrap(open_badger_memory()?, 1, None))
}

fn open_namespace_chunk() -> anyhow::Result<Chunk<PrefixKey<BadgerDb>>> {
    Ok(Chunk::wrap(open_namespace()?, 1, None))
}

fn open_chunk_namespace() -> anyhow::Result<PrefixKey<Chunk<BadgerDb>>> {
    Ok(PrefixKey::wrap(open_chunk()?, b"/example".to_vec()))
}

macro_rules! badger_db_test {
    ($test_fn:ident) => {
        mod $test_fn {
            #[test]
            fn badger_memory() -> anyhow::Result<()> {
                super::$test_fn(crate::open_badger_memory()?);
                Ok(())
            }

            #[test]
            fn namespace() -> anyhow::Result<()> {
                super::$test_fn(crate::open_namespace()?);
                Ok(())
            }

            #[test]
            fn chunk() -> anyhow::Result<()> {
                super::$test_fn(crate::open_chunk()?);
                Ok(())
            }

            #[test]
            fn namespace_chunk() -> anyhow::Result<()> {
                super::$test_fn(crate::open_namespace_chunk()?);
                Ok(())
            }

            #[test]
            fn chunk_namespace() -> anyhow::Result<()> {
                super::$test_fn(crate::open_chunk_namespace()?);
                Ok(())
            }
        }
    };
}

pub(crate) use badger_db_test;
