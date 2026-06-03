
#[cfg(test)]
mod snapshot {
    use corekv::{
        BadgerDb, BadgerSnapshotIterError, Iter, IterOptions, OpenOptions, ReaderWriterIter, Writer,
    };

    fn db_opts() -> OpenOptions {
        OpenOptions::builder().in_memory(true).build()
    }

    #[test]
    fn test_db_iter_trait() {
        let mut db = BadgerDb::new("", db_opts()).expect("opening badgerdb failed");

        // iter that yields nothing back should not error just return Ok(false)
        let mut null_iter = db
            .iter(IterOptions::builder().prefix(b"doesnotexist").build())
            .expect("faild to create iter");
        assert!(!null_iter.next().expect("next failed on no-yield iter"));
        assert!(!null_iter.next().expect("next failed on no-yield iter"));

        for key in 0..2 {
            for end in 0..2 {
                let db_key = format!("key{key}_end{end}");
                let db_val = format!("root{key}_{end}");
                db.set(db_key.as_bytes(), db_val.as_bytes())
                    .expect("db set failed");
            }
        }

        // again, now with data: iter that yields nothing back should not erro just return Ok(false)
        let mut null_iter = db
            .iter(IterOptions::builder().prefix(b"doesnotexist").build())
            .expect("faild to create iter");
        assert!(!null_iter.next().expect("next failed on no-yield iter"));
        assert!(!null_iter.next().expect("next failed on no-yield iter"));

        // should yield all values, if next isn't called it should yield SnapshotIterError::NoEntrySelected
        let mut iter = db
            .iter(IterOptions::default())
            .expect("creating iter failed");
        assert!(matches!(
            iter.key(),
            Err(BadgerSnapshotIterError::NoEntrySelected)
        ));
        assert!(matches!(
            iter.value(),
            Err(BadgerSnapshotIterError::NoEntrySelected)
        ));

        for key in 0..2 {
            for end in 0..2 {
                let db_key = format!("key{key}_end{end}");
                let db_val = format!("root{key}_{end}");
                assert!(iter.next().expect("iter next failed"));
                assert_eq!(iter.key().expect("iter key failed"), db_key.as_bytes());
                assert_eq!(iter.value().expect("iter value failed"), db_val.as_bytes());
            }
        }

        // seek last key1_end_0 which should yield key1_end0 and key1_end1
        assert!(iter.seek(b"key1_end0").expect("iter seek failed"));
        assert_eq!(iter.key().expect("iter key failed"), b"key1_end0".to_vec());
        assert_eq!(
            iter.value().expect("iter value failed"),
            b"root1_0".to_vec()
        );
        assert!(iter.next().expect("iter next failed"));
        assert_eq!(iter.key().expect("iter key failed"), b"key1_end1".to_vec());
        assert_eq!(
            iter.value().expect("iter value failed"),
            b"root1_1".to_vec()
        );
        assert!(!iter.next().expect("iter next failed"));

        // reset iter, this is identical to creating a new iter (not same as seek first key)
        iter.reset().expect("iter reset failed");
        assert!(matches!(
            iter.key(),
            Err(BadgerSnapshotIterError::NoEntrySelected)
        ));
        assert!(matches!(
            iter.value(),
            Err(BadgerSnapshotIterError::NoEntrySelected)
        ));

        for key in 0..2 {
            for end in 0..2 {
                let db_key = format!("key{key}_end{end}");
                let db_val = format!("root{key}_{end}");
                assert!(iter.next().expect("iter next failed"));
                assert_eq!(iter.key().expect("iter key failed"), db_key.as_bytes());
                assert_eq!(iter.value().expect("iter value failed"), db_val.as_bytes());
            }
        }

        // reverse check
        let mut iter_rev = db
            .iter(IterOptions::builder().reverse(true).build())
            .expect("creating iter failed");
        for key in (0..2).rev() {
            for end in (0..2).rev() {
                let db_key = format!("key{key}_end{end}");
                let db_val = format!("root{key}_{end}");
                assert!(iter_rev.next().expect("iter next failed"));
                assert_eq!(iter_rev.key().expect("iter key failed"), db_key.as_bytes());
                assert_eq!(
                    iter_rev.value().expect("iter value failed"),
                    db_val.as_bytes()
                );
            }
        }
    }

    #[test]
    fn test_db_iter_opts() {
        let mut db = BadgerDb::new("", db_opts()).expect("opening badgerdb failed");

        for key in 0..2 {
            for end in 0..2 {
                let db_key = format!("key{key}_end{end}");
                let db_val = format!("root{key}_{end}");
                db.set(db_key.as_bytes(), db_val.as_bytes())
                    .expect("db set failed");
            }
        }

        // prefix only
        let mut db_iter_key_prefix = db
            .iter(IterOptions::builder().prefix(b"key0").build())
            .expect("failed to create iter");

        for end in 0..2 {
            let db_key = format!("key0_end{end}");
            let db_val = format!("root0_{end}");
            assert!(db_iter_key_prefix.next().expect("iter next failed"));
            assert_eq!(
                db_iter_key_prefix.key().expect("iter key failed"),
                db_key.as_bytes()
            );
            assert_eq!(
                db_iter_key_prefix.value().expect("iter value failed"),
                db_val.as_bytes()
            );
        }

        // key_range_first only
        let mut db_iter_key_start = db
            .iter(IterOptions::builder().key_range_start(b"key1").build())
            .expect("failed to create iter");

        for end in 0..2 {
            let db_key = format!("key1_end{end}");
            let db_val = format!("root1_{end}");
            assert!(db_iter_key_start.next().expect("iter next failed"));
            assert_eq!(
                db_iter_key_start.key().expect("iter key failed"),
                db_key.as_bytes()
            );
            assert_eq!(
                db_iter_key_start.value().expect("iter value failed"),
                db_val.as_bytes()
            );
        }

        // key_range_end only
        let mut db_iter_key_end = db
            .iter(IterOptions::builder().key_range_end(b"key1").build())
            .expect("failed to create iter");

        for end in 0..2 {
            let db_key = format!("key0_end{end}");
            let db_val = format!("root0_{end}");
            assert!(db_iter_key_end.next().expect("iter next failed"));
            assert_eq!(
                db_iter_key_end.key().expect("iter key failed"),
                db_key.as_bytes()
            );
            assert_eq!(
                db_iter_key_end.value().expect("iter value failed"),
                db_val.as_bytes()
            );
        }
        assert!(!db_iter_key_end.next().expect("iter next failed"));

        // key_range_start: key1_end0; key_range_end: key1_end0 should yield nothing
        let mut db_iter_key_range = db
            .iter(
                IterOptions::builder()
                    .key_range_start(b"key1_end0")
                    .key_range_end(b"key1_end0")
                    .build(),
            )
            .expect("failed to create iter");
        assert!(!db_iter_key_range.next().expect("iter next failed"));

        // key_range_start: key1_end0; key_range_end: key1_end1 should yield key1_end0 only since key_range_end is not included
        let mut db_iter_key_range = db
            .iter(
                IterOptions::builder()
                    .key_range_start(b"key1_end0")
                    .key_range_end(b"key1_end1")
                    .build(),
            )
            .expect("failed to create iter");
        assert!(db_iter_key_range.next().expect("iter next failed"));
        assert_eq!(
            db_iter_key_range.key().expect("iter key failed"),
            b"key1_end0"
        );
        assert_eq!(
            db_iter_key_range.value().expect("iter value failed"),
            b"root1_0"
        );
        assert!(!db_iter_key_range.next().expect("iter next failed"));

        // IMPORTANT: for some reason when prefix is used range_end is completely ignored
        // prefix: key1 + key_range_end: key1_end1 should yield key1_end0
        let mut db_iter_key_prefix_and_end = db
            .iter(
                IterOptions::builder()
                    .prefix(b"key1")
                    .key_range_end(b"key1_end1")
                    .build(),
            )
            .expect("failed to create iter");
        assert!(db_iter_key_prefix_and_end.next().expect("iter next failed"));
        assert_eq!(
            db_iter_key_prefix_and_end.key().expect("iter key failed"),
            b"key1_end0"
        );
        assert_eq!(
            db_iter_key_prefix_and_end
                .value()
                .expect("iter value failed"),
            b"root1_0"
        );

        // this should fail if key_range_end would be respected while prefix is set
        assert!(db_iter_key_prefix_and_end.next().expect("iter next failed"));
        assert_eq!(
            db_iter_key_prefix_and_end.key().expect("iter key failed"),
            b"key1_end1"
        );
        assert!(!db_iter_key_prefix_and_end.next().expect("iter next failed"));

        // keys_only and reverse
        let mut db_iter_keys_only = db
            .iter(IterOptions::builder().keys_only(true).reverse(true).build())
            .expect("failed to create iter");

        for key in (0..2).rev() {
            for end in (0..2).rev() {
                let db_key = format!("key{key}_end{end}");
                assert!(db_iter_keys_only.next().expect("iter next failed"));
                assert_eq!(
                    db_iter_keys_only.key().expect("iter key failed"),
                    db_key.as_bytes()
                );
                assert!(matches!(
                    db_iter_keys_only.value(),
                    Err(BadgerSnapshotIterError::KeysOnly)
                ));
            }
        }
    }
}
