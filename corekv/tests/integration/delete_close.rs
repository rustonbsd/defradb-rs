use corekv::{Db, Snapshot};

use crate::{State, get_base_error, tests};

fn test_close_then_delete<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.db.close();
    let err_msg =
        get_base_error(&state.delete(b"not important").expect_err("should error")).to_string();
    assert!(err_msg.ends_with("db is closed") || err_msg.ends_with("Database closed"));
}

tests!(test_close_then_delete: db);
