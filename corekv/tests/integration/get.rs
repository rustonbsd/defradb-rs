use crate::{DbTest, badger_db_test};

fn test_get<D>(db: D)
where
    D: DbTest,
{
    assert!(
        db.get(b"not important")
            .expect("empty key should not error")
            .is_none()
    );
    db.close()
}

badger_db_test!(test_get);
