use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_set_get<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    assert!(state.get(b"k1").expect("get k1").as_deref() == Some(b"v1"));
    state
}

fn test_set_get_multiple<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k2", b"").expect("set k2");
    state.set(b"k3", b"v3").expect("set k3");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    assert!(state.get(b"k1").expect("get k1").as_deref() == Some(b"v1"));
    assert!(state.get(b"k3").expect("get k3").as_deref() == Some(b"v3"));
    assert!(state.get(b"k2").expect("get k2").as_deref() == Some(b""));
    assert!(state.get(b"k1").expect("get k1").as_deref() == Some(b"v1"));
    state
}

tests!(test_set_get: db + snapshot);
tests!(test_set_get_multiple: db + snapshot);
