use crate::{DbTest, badger_db_test};

fn test_set_get<D>(mut db: D)
where
    D: DbTest,
{
    db.set(b"k1", b"v1").expect("set should succeed");
    assert!(db.get(b"k1").expect("get should succeed").as_deref() == Some(b"v1"));
    db.close()
}

fn test_set_get_multiple<D>(mut db: D)
where
    D: DbTest,
{
    db.set(b"k1", b"v1").expect("set should succeed");
    db.set(b"k2", b"").expect("set should succeed");
    db.set(b"k3", b"v3").expect("set should succeed");

    assert!(db.get(b"k1").expect("get should succeed").as_deref() == Some(b"v1"));
    assert!(db.get(b"k3").expect("get should succeed").as_deref() == Some(b"v3"));
    assert!(db.get(b"k2").expect("get should succeed").is_none());
    assert!(db.get(b"k1").expect("get should succeed").as_deref() == Some(b"v1"));
    db.close()
}

badger_db_test!(test_set_get);
badger_db_test!(test_set_get_multiple);
