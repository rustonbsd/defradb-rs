use crate::{DbTest, badger_db_test, get_base_error};

fn test_close_then_has<D>(db: D)
where
    D: DbTest,
{
    db.close();
    assert!(
        get_base_error(&db.has(b"not important").expect_err("should error"))
            .to_string()
            .ends_with("db is closed")
    );
}

badger_db_test!(test_close_then_has);
