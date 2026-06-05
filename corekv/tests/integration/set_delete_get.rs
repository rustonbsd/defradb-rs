use crate::{DbTest, badger_db_test};

fn test_set_delete_get<D>(mut db: D)
where
    D: DbTest,
{
    db.set(b"k1", b"v1").expect("set should succeed");
    assert!(db.get(b"k1").expect("get should succeed").as_deref() == Some(b"v1"));
    db.delete(b"k1").expect("delete should succeed");
    assert!(db.get(b"k1").expect("empty key should not error").is_none());
    db.close()
}

badger_db_test!(test_set_delete_get);
