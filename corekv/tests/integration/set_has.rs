use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_set_has<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"does not matter").expect("set k1");
    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    assert!(state.has(b"k1").expect("has k1"));
    state
}

tests!(test_set_has: db + snapshot);
