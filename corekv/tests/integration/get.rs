use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_get<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    assert!(
        state
            .get(b"not important")
            .expect("empty key should not error")
            .is_none()
    );
    state.db.close()
}

tests!(test_get; db, snapshot);
