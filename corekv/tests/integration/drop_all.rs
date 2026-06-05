use crate::{DbTest, badger_db_test};

fn test_drop_all<D>(db: D)
where
    D: DbTest,
{
    assert!(db.drop_all().is_ok());
    db.close()
}

badger_db_test!(test_drop_all);
