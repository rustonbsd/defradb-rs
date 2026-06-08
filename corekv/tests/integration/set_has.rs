use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_set_has<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state
        .set(b"k1", b"does not matter")
        .expect("set should succeed");
    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    assert!(state.has(b"k1").expect("should succeed"));
}

tests!(test_set_has; db, snapshot);
