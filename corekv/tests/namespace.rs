#[cfg(test)]
mod namespace {
    use corekv::{
        BadgerDb, Iter, IterOptions, OpenOptions, PrefixKey, Reader, ReaderWriterIter, Writer,
    };

    fn db_opts() -> OpenOptions {
        OpenOptions::builder().in_memory(true).build()
    }

    #[test]
    fn test_db_set_namespace() {
        let mut db = BadgerDb::new("", db_opts()).expect("opening badgerdb failed");
        let mut ns_db = PrefixKey::wrap(db.clone(), b"ns1:".to_vec());

        db.set(b"key0", b"root0").expect("root store setting key0");
        ns_db.set(b"key0", b"ns0").expect("setting key0 failed");

        assert_eq!(
            db.get(b"key0")
                .expect("getting key0 from root store failed"),
            Some(b"root0".to_vec())
        );
        assert_eq!(
            ns_db.get(b"key0").expect("getting key0 from ns_db failed"),
            Some(b"ns0".to_vec())
        );

        ns_db.set(b"key1", b"ns1").expect("setting key1 failed");
        db.set(b"key1", b"root1")
            .expect("setting key1 in root store failed");

        assert_eq!(
            db.get(b"key1")
                .expect("getting key1 from root store failed"),
            Some(b"root1".to_vec())
        );
        assert_eq!(
            ns_db.get(b"key1").expect("getting key1 from ns_db failed"),
            Some(b"ns1".to_vec())
        );

        // check that keys are properly namespaced
        assert_eq!(
            db.get(b"ns1:key0")
                .expect("getting namespaced key0 from root store failed"),
            Some(b"ns0".to_vec())
        );
        assert_eq!(
            db.get(b"ns1:key1")
                .expect("getting namespaced key1 from root store failed"),
            Some(b"ns1".to_vec())
        );
    }

    #[test]
    fn test_db_iter_namespace() {
        let mut db = BadgerDb::new("", db_opts()).expect("opening badgerdb failed");
        let mut ns = PrefixKey::wrap(db.clone(), b"ns1:".to_vec());

        for key in 0..2 {
            for end in 0..2 {
                let db_key = format!("key{key}_end{end}");
                let db_val = format!("root{key}_{end}");
                let ns_key = db_key.clone();
                let ns_val = format!("ns{key}_{end}");
                db.set(db_key.as_bytes(), db_val.as_bytes())
                    .expect("db set failed");
                ns.set(ns_key.as_bytes(), ns_val.as_bytes())
                    .expect("ns_db set failed");
            }
        }

        // prefix: key; check that all keys are returned as expected for both db and ns while reverse is set
        let mut db_iter = db
            .iter(IterOptions::builder().prefix(b"key").reverse(true).build())
            .expect("db iter failed");
        let mut ns_iter = ns
            .iter(IterOptions::builder().prefix(b"key").reverse(true).build())
            .expect("ns iter failed");

        for key in (0..2).rev() {
            for end in (0..2).rev() {
                let db_key = format!("key{key}_end{end}");
                let db_val = format!("root{key}_{end}");
                let ns_key = db_key.clone();
                let ns_val = format!("ns{key}_{end}");

                assert!(db_iter.next().expect("db iter next failed"));
                assert_eq!(
                    db_iter.key().expect("db iter key failed"),
                    db_key.as_bytes()
                );
                assert_eq!(
                    db_iter.value().expect("db iter value failed"),
                    db_val.as_bytes()
                );

                assert!(ns_iter.next().expect("ns iter next failed"));
                assert_eq!(
                    ns_iter.key().expect("ns iter key failed"),
                    ns_key.as_bytes()
                );
                assert_eq!(
                    ns_iter.value().expect("ns iter value failed"),
                    ns_val.as_bytes()
                );
            }
        }
        assert!(!db_iter.next().expect("db iter next failed"));
        assert!(!ns_iter.next().expect("ns iter next failed"));

        // no prefix db read should yield all db keys and ns keys
        let mut db_iter_no_prefix = db
            .iter(IterOptions::builder().build())
            .expect("db iter no prefix failed");
        // order is determined by time written since k < n so if it was based on ascii it should be the other way around
        for key in 0..2 {
            for end in 0..2 {
                let db_key = format!("key{key}_end{end}");
                let db_val = format!("root{key}_{end}");
                assert!(
                    db_iter_no_prefix
                        .next()
                        .expect("db iter no prefix next failed")
                );
                assert_eq!(
                    db_iter_no_prefix
                        .key()
                        .expect("db iter no prefix key failed"),
                    db_key.as_bytes()
                );
                assert_eq!(
                    db_iter_no_prefix
                        .value()
                        .expect("db iter no prefix value failed"),
                    db_val.as_bytes()
                );
            }
        }

        for key in 0..2 {
            for end in 0..2 {
                let ns_key = format!("ns1:key{key}_end{end}");
                let ns_val = format!("ns{key}_{end}");
                assert!(
                    db_iter_no_prefix
                        .next()
                        .expect("db iter no prefix next failed")
                );
                assert_eq!(
                    db_iter_no_prefix
                        .key()
                        .expect("db iter no prefix key failed"),
                    ns_key.as_bytes()
                );
                assert_eq!(
                    db_iter_no_prefix
                        .value()
                        .expect("db iter no prefix value failed"),
                    ns_val.as_bytes()
                );
            }
        }
        assert!(
            !db_iter_no_prefix
                .next()
                .expect("db iter no prefix next failed")
        );
    }
}
