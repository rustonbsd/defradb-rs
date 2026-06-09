mod close;

mod delete_close;
mod drop_all;
mod get;
mod get_close;
mod has;
mod has_close;
mod iterator;
mod set_close;
mod set_delete_get;
mod set_drop_all_has;
mod set_get;
mod set_has;

use std::error::Error;

use corekv::{BadgerDb, Chunk, Db, Iter, IterOptions, OpenOptions, PrefixKey, Snapshot};

enum Active<D, S>
where
    D: Db<Snapshot = S>,
    S: Snapshot,
{
    Db(D),
    Snapshot(S),
}
struct State<D, S>
where
    D: Db<Snapshot = S>,
    S: Snapshot,
{
    db: D,
    current: Active<D, S>,
    multi_snap: Option<S>,
    commit_after_writes: bool,
}

pub fn format_error_chain(err: &Box<dyn Error>) -> String {
    let mut out = err.to_string();
    let mut current = err.source();

    while let Some(source) = current {
        out.push_str(" -> ");
        out.push_str(&source.to_string());
        current = source.source();
    }

    out
}

fn get_base_error(err: &Box<dyn Error>) -> &dyn Error {
    let mut current = err.as_ref();

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

fn plain_start<D>(db: D) -> State<D, D::Snapshot>
where
    D: Db,
{
    State {
        db: db.clone(),
        current: Active::Db(db.clone()),
        multi_snap: None,
        commit_after_writes: false,
    }
}

fn plain_end<D, S>(state: State<D, S>)
where
    D: Db<Snapshot = S>,
    S: Snapshot,
{
    state.db.close();
}

// create two snapshots, apply everything to the main snapshot, before we discard both snapshots at the end
// we assert that the second untouched snapshot is still empty via an iterator
fn snapshot_multi_start<D>(db: D) -> State<D, D::Snapshot>
where
    D: Db,
{
    State {
        db: db.clone(),
        current: Active::Snapshot(db.create_read_write_snapshot().expect("create snapshot")),
        multi_snap: Some(db.create_read_write_snapshot().expect("create snapshot")),
        commit_after_writes: false,
    }
}

fn snapshot_multi_end<D, S>(state: State<D, S>)
where
    D: Db<Snapshot = S>,
    S: Snapshot,
{
    if state.db.is_closed() {
        log::info!(
            "db.close() was called as part of the test, we have to exclude this variant from the multi test"
        );
        return;
    }
    if let Some(snap) = state.multi_snap {
        let mut iter = snap.iter(IterOptions::default()).expect("create iter");
        assert!(!iter.next().expect("no next item available"));
        iter.close().expect("close iter");
        snap.discard();
    } else {
        unreachable!("current state is only allowed to be a snapshot");
    }

    if let Active::Snapshot(snap) = state.current {
        snap.discard();
    } else {
        unreachable!("current state is only allowed to be a snapshot");
    }
    state.db.close();
}

// discard at end then read db, assert db has no changes
fn snapshot_discard_start<D>(db: D) -> State<D, D::Snapshot>
where
    D: Db,
{
    State {
        db: db.clone(),
        current: Active::Snapshot(db.create_read_write_snapshot().expect("create snapshot")),
        multi_snap: None,
        commit_after_writes: false,
    }
}

fn snapshot_discard_end<D, S>(state: State<D, S>)
where
    D: Db<Snapshot = S>,
    S: Snapshot,
{
    if state.db.is_closed() {
        log::info!(
            "db.close() was called as part of the test, we have to exclude this variant from the multi test"
        );
        return;
    }
    let mut iter = state.db.iter(IterOptions::default()).expect("create iter");
    assert!(!iter.next().expect("no next item avaliable"));
    iter.close().expect("close iter");
    if let Active::Snapshot(snap) = state.current {
        snap.discard();
    } else {
        unreachable!("current state is only allowed to be a snapshot");
    }
    state.db.close();
}

// we call state.commit_after_writes() after all writes are done and replace current with db to perform the rest of the reads
// (as far as I can tell all write operations come before all read operations in all tests)
fn snapshot_commit_start<D>(db: D) -> State<D, D::Snapshot>
where
    D: Db,
{
    State {
        db: db.clone(),
        current: Active::Snapshot(db.create_read_write_snapshot().expect("create snapshot")),
        multi_snap: None,
        commit_after_writes: true,
    }
}

fn snapshot_commit_end<D, S>(state: State<D, S>)
where
    D: Db<Snapshot = S>,
    S: Snapshot,
{
    state.db.close();
}

impl<D, S> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Box<dyn Error>> {
        match &mut self.current {
            Active::Db(db) => db.set(key, value).map_err(Into::into),
            Active::Snapshot(snap) => snap.set(key, value).map_err(Into::into),
        }
    }

    fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
        match &mut self.current {
            Active::Db(db) => db.get(key).map_err(Into::into),
            Active::Snapshot(snap) => snap.get(key).map_err(Into::into),
        }
    }

    fn has(&mut self, key: &[u8]) -> Result<bool, Box<dyn Error>> {
        match &mut self.current {
            Active::Db(db) => db.has(key).map_err(Into::into),
            Active::Snapshot(snap) => snap.has(key).map_err(Into::into),
        }
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), Box<dyn Error>> {
        match &mut self.current {
            Active::Db(db) => db.delete(key).map_err(Into::into),
            Active::Snapshot(snap) => snap.delete(key).map_err(Into::into),
        }
    }

    fn commit_after_writes(&mut self) -> Result<(), Box<dyn Error>> {
        if self.commit_after_writes {
            self.commit()?;
            self.current = Active::Db(self.db.clone());
        }
        Ok(())
    }

    fn commit(&mut self) -> Result<(), Box<dyn Error>> {
        match &mut self.current {
            Active::Db(_) => unreachable!("commit can only be called on snapshots"),
            Active::Snapshot(snap) => snap.commit().map_err(Into::into),
        }
    }

    #[allow(dead_code)]
    fn discard(&mut self) {
        match &mut self.current {
            Active::Db(_) => unreachable!("discard can only be called on snapshots"),
            Active::Snapshot(snap) => snap.discard(),
        }
    }

    fn drop_all(&mut self) -> Result<(), Box<dyn Error>> {
        match &mut self.current {
            Active::Db(db) => db.drop_all().map_err(Into::into),
            Active::Snapshot(_) => {
                unreachable!("snapshot tests not supported (drop all only on db)")
            }
        }
    }

    fn iter(&mut self, opts: IterOptions) -> Result<S::Iter, Box<dyn Error>> {
        match &mut self.current {
            Active::Db(db) => db.iter(opts).map_err(Into::into),
            Active::Snapshot(snap) => snap.iter(opts).map_err(Into::into),
        }
    }
}

#[macro_export]
macro_rules! test_cases {
    ($variant:ident, $test_fn:ident, $start_fn:path, $end_fn:path) => {
        mod $variant {
            #[test]
            fn badger_memory() -> anyhow::Result<()> {
                let db = $crate::open_badger_memory()?;
                let mut state = $start_fn(db);
                super::super::$test_fn(&mut state);
                $end_fn(state);
                Ok(())
            }

            #[test]
            fn namespace() -> anyhow::Result<()> {
                let db = $crate::open_namespace()?;
                let mut state = $start_fn(db);
                super::super::$test_fn(&mut state);
                $end_fn(state);
                Ok(())
            }

            #[test]
            fn chunk() -> anyhow::Result<()> {
                let db = $crate::open_chunk()?;
                let mut state = $start_fn(db);
                super::super::$test_fn(&mut state);
                $end_fn(state);
                Ok(())
            }

            #[test]
            fn namespace_chunk() -> anyhow::Result<()> {
                let db = $crate::open_namespace_chunk()?;
                let mut state = $start_fn(db);
                super::super::$test_fn(&mut state);
                $end_fn(state);
                Ok(())
            }

            #[test]
            fn chunk_namespace() -> anyhow::Result<()> {
                let db = $crate::open_chunk_namespace()?;
                let mut state = $start_fn(db);
                super::super::$test_fn(&mut state);
                $end_fn(state);
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! db_body {
    ($test_fn:ident) => {
        mod db {
            $crate::test_cases!(plain, $test_fn, $crate::plain_start, $crate::plain_end);
        }
    };
}

#[macro_export]
macro_rules! snapshot_body {
    ($test_fn:ident) => {
        mod snapshot {
            $crate::test_cases!(
                snapshot_discard,
                $test_fn,
                $crate::snapshot_discard_start,
                $crate::snapshot_discard_end
            );
            $crate::test_cases!(
                snapshot_commit,
                $test_fn,
                $crate::snapshot_commit_start,
                $crate::snapshot_commit_end
            );
            $crate::test_cases!(
                snapshot_multi,
                $test_fn,
                $crate::snapshot_multi_start,
                $crate::snapshot_multi_end
            );
        }
    };
}

#[macro_export]
macro_rules! tests {
    ($test_fn:ident: $($rest:tt)+) => {
        mod $test_fn {
            use super::$test_fn;
            $crate::__tests_variants!($test_fn; $($rest)+);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __tests_variants {
    ($test_fn:ident; $variant:ident $(+)?) => {
        $crate::__tests_body!($variant, $test_fn);
    };

    ($test_fn:ident; $variant:ident + $($rest:tt)+) => {
        $crate::__tests_body!($variant, $test_fn);
        $crate::__tests_variants!($test_fn; $($rest)+);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __tests_body {
    (db, $test_fn:ident) => {
        $crate::db_body!($test_fn);
    };
    (snapshot, $test_fn:ident) => {
        $crate::snapshot_body!($test_fn);
    };
}
