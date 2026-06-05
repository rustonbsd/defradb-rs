use crate::{DbTest, badger_db_test};

fn test_has<D>(db: D)
where
    D: DbTest,
{
    assert!(!db.has(b"not important").expect("has failed"));
    db.close()
}

badger_db_test!(test_has);
