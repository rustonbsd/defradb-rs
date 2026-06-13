use corekv::{Db, Iter, IterOptions, Snapshot};

use crate::{State, get_base_error, tests};

fn test_close_next<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    let mut iter = state.iter(IterOptions::default()).expect("create iter");
    state.db.close();
    assert!(
        get_base_error(&iter.next().expect_err("next on closed iter errors"))
            .to_string()
            .ends_with("Database closed")
    );
    state
}

tests!(test_close_next: db + snapshot);
