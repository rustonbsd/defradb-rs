use crate::{DbTest, badger_db_test};

fn test_set_has<D>(mut db: D)
where
    D: DbTest,
{
    db.set(b"k1", b"does not matter")
        .expect("set should succeed");
    assert!(db.has(b"k1").expect("should succeed"));
    db.close()
}

badger_db_test!(test_set_has);
