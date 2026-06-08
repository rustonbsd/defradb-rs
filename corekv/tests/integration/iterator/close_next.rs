use corekv::{Db, Iter, IterOptions, Snapshot};

use crate::{State, get_base_error, tests};

fn test_close_next<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    let mut iter = state
        .iter(IterOptions::default())
        .expect("failed to create iterator");
    state.db.close();
    assert!(
        get_base_error(
            &Box::new(iter.next().expect_err("next on closed iter expect error")).into()
        )
        .to_string()
        .ends_with("Database closed")
    );
}

tests!(test_close_next; db, snapshot);
