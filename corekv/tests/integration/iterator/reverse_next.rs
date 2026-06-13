use crate::{State, tests};
use corekv::{Db, Iter, IterOptions, Snapshot};

fn test_reverse_next<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(IterOptions::builder().reverse(true).build())
        .expect("create iter");

    assert!(iter.next().expect("yield next item"));

    iter.close().expect("close iter");
    state
}

fn test_reverse_next_beyond_end<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k3", b"").expect("set k3");
    state.set(b"k4", b"v4").expect("set k4");
    state.set(b"k2", b"v2").expect("set k2");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(IterOptions::builder().reverse(true).build())
        .expect("create iter");

    assert!(iter.next().expect("yield next item"));
    assert!(iter.next().expect("yield next item"));
    assert!(iter.next().expect("yield next item"));
    assert!(iter.next().expect("yield next item"));

    assert!(!iter.next().expect("yields no next item"));
    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

tests!(test_reverse_next: db + snapshot);
tests!(test_reverse_next_beyond_end: db + snapshot);
