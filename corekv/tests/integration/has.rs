use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_has<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    assert!(!state.has(b"not important").expect("returns false"));
    state
}

tests!(test_has: db + snapshot);
