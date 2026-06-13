use crate::{State, tests};
use corekv::{Db, Iter, IterOptions, Snapshot};

fn test_prefix<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"1", b"v1").expect("set k1");
    state.set(b"k3", b"").expect("overwrite k3");
    state.set(b"4", b"v4").expect("overwrite k4");
    state.set(b"k2", b"v2").expect("set k2");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(IterOptions::builder().prefix(b"k").build())
        .expect("create iter");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k2");
    assert_eq!(iter.value().expect("get value"), b"v2");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k3");
    assert_eq!(iter.value().expect("get value"), b"");

    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

fn test_prefix_does_not_return_self<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k", b"v").expect("overwrite k");
    state.set(b"k1", b"v1").expect("set k1");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(IterOptions::builder().prefix(b"k").build())
        .expect("create iter");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k");
    assert_eq!(iter.value().expect("get value"), b"v");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k1");
    assert_eq!(iter.value().expect("get value"), b"v1");

    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

tests!(test_prefix: db + snapshot | chunk);
tests!(test_prefix_does_not_return_self: db + snapshot | chunk);
