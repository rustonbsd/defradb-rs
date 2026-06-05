use crate::{DbTest, badger_db_test};

fn test_set_drop_all_has<D>(mut db: D)
where
    D: DbTest,
{
    db.set(b"k1", b"v1").expect("set should succeed");
    db.set(b"k2", b"v2").expect("set should succeed");
    assert!(db.has(b"k1").expect("has should succeed"));
    assert!(db.has(b"k2").expect("has should succeed"));
    db.drop_all().expect("drop_all should succeed");
    assert!(!db.has(b"k1").expect("has should succeed"));
    assert!(!db.has(b"k2").expect("has should succeed"));
    db.close()
}

badger_db_test!(test_set_drop_all_has);
