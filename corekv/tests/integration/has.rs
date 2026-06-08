use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_has<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    assert!(!state.has(b"not important").expect("has failed"));
}

tests!(test_has; db, snapshot);
