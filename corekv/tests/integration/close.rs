use crate::{DbTest, badger_db_test};

fn test_close<D>(db: D)
where
    D: DbTest,
{
    db.close();
}

fn test_close_twice<D>(db: D)
where
    D: DbTest,
{
    db.close();
    db.close();
}

badger_db_test!(test_close);
badger_db_test!(test_close_twice);
