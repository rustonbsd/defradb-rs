use corekv::{Db, IterOptions, Snapshot};

use crate::{State, get_base_error, tests};

fn test_close_iter<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.db.close();
    let err_msg = get_base_error(&Box::new(
        state
            .iter(IterOptions::default())
            .expect_err("iter on closed db expect error"),
    ))
    .to_string();
    assert!(err_msg.ends_with("Database closed") || err_msg.ends_with("badger db is closed"));
}

tests!(test_close_iter: db + snapshot);
