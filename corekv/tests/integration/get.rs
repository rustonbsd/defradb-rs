use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_get<D, S>(mut state: State<D, S>) -> State<D, S>
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
            .expect("returns empty value: None")
            .is_none()
    );
    state.db.close();
    state
}

tests!(test_get: db + snapshot);
