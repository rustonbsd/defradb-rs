use corekv::{Db, IterOptions, Snapshot};

use crate::{State, get_base_error, tests};

fn test_close_iter<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.db.close();
    let err_msg = get_base_error(
        state
            .iter(IterOptions::default())
            .expect_err("iter on closed db expect error")
            .as_ref(),
    )
    .to_string();
    assert!(err_msg.ends_with("Database closed") || err_msg.ends_with("badger db is closed"));
    state
}

tests!(test_close_iter: db + snapshot);
