// todo: concurrent snapshot commits (iron out exact commit behaviour in all cases)
// todo: read ony snapshot blocks all write fn calls
// todo: make sure no ffi panics are possible in non standard usage (eg snapshot drop behaviour, still open snapshots after db is closed)

mod common;

#[cfg(test)]
mod snapshot {

    use corekv::{
        BadgerDb, BadgerSnapshotError, BadgerSnapshotIterError, Iter, IterOptions, Reader,
        ReaderWriterIter, Snapshot, SnapshotCreator, Writer,
    };

    use crate::common::db_opts;

    #[test]
    fn test_snapshot_iter() {
        let db = BadgerDb::new("", db_opts()).expect("opening badgerdb failed");
        let mut snapshot = db
            .create_read_write_snapshot()
            .expect("creating snapshot failed");

        for key in 0..4 {
            snapshot
                .set(
                    format!("key{key}").as_bytes(),
                    format!("value{key}").as_bytes(),
                )
                .expect("setting snapshot key failed");
        }

        let mut iter = snapshot
            .iter(IterOptions::default())
            .expect("creating snapshot iterator failed");
        for key in 0..4 {
            assert!(iter.next().expect("iterating snapshot failed"));
            assert_eq!(
                iter.key().expect("getting key failed"),
                format!("key{key}").as_bytes()
            );
            assert_eq!(
                iter.value().expect("getting value failed"),
                format!("value{key}").as_bytes()
            );
        }
        assert!(!iter.next().expect("iterating snapshot failed"));

        iter.close().expect("closing snapshot iterator failed");
        assert!(matches!(
            iter.next(),
            Err(BadgerSnapshotIterError::IterClosed)
        ));
    }

    #[test]
    fn test_snapshot_discard() {
        let db = BadgerDb::new("", db_opts()).expect("opening badgerdb failed");
        let mut snapshot = db
            .create_read_write_snapshot()
            .expect("creating snapshot failed");

        snapshot
            .set(b"key", b"value")
            .expect("setting snapshot key failed");
        assert_eq!(
            snapshot.get(b"key").expect("getting snapshot key failed"),
            Some(b"value".to_vec())
        );

        snapshot.discard();

        assert!(matches!(
            snapshot.get(b"key"),
            Err(BadgerSnapshotError::Discarded)
        ));
        assert!(matches!(
            snapshot.has(b"key"),
            Err(BadgerSnapshotError::Discarded)
        ));
        assert!(matches!(
            snapshot.iter(IterOptions::default()),
            Err(BadgerSnapshotError::Discarded)
        ));
    }

    #[test]
    fn test_snapshot_commit() {
        let db = BadgerDb::new("", db_opts()).expect("opening badgerdb failed");
        let mut snapshot = db
            .create_read_write_snapshot()
            .expect("creating snapshot failed");

        snapshot
            .set(b"key", b"value")
            .expect("setting snapshot key failed");
        assert_eq!(
            snapshot.get(b"key").expect("getting snapshot key failed"),
            Some(b"value".to_vec())
        );

        snapshot.commit().expect("committing snapshot failed");

        assert_eq!(
            db.get(b"key").expect("getting db key failed"),
            Some(b"value".to_vec())
        );

        // commit is treated same as discarded: no more operations allowed on snapshot
        assert!(matches!(
            snapshot.get(b"key"),
            Err(BadgerSnapshotError::Discarded)
        ));
    }
}
