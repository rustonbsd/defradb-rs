use crate::{State, tests};
use corekv::{Db, Iter, IterOptions, Snapshot};

fn test_delete<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k3", b"").expect("set k3");
    state.set(b"k4", b"v4").expect("set k4");
    state.set(b"k2", b"v2").expect("set k2");
    state.delete(b"k2").expect("delete k2");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state.iter(IterOptions::default()).expect("create iter");

    iter.next().expect("yields next item");
    assert_eq!(iter.key().expect("get key"), b"k1");
    assert_eq!(iter.value().expect("get value"), b"v1");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k3");
    assert_eq!(iter.value().expect("get value"), b"");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k4");
    assert_eq!(iter.value().expect("get value"), b"v4");

    iter.close().expect("close iter");
    state
}

fn test_delete_last_item<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k3", b"").expect("set k3");
    state.set(b"k4", b"v4").expect("set k4");
    state.set(b"k2", b"v2").expect("set k2");
    state.delete(b"k4").expect("delete k4");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state.iter(IterOptions::default()).expect("create iter");

    iter.next().expect("yields next item");
    assert_eq!(iter.key().expect("get key"), b"k1");
    assert_eq!(iter.value().expect("get value"), b"v1");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k2");
    assert_eq!(iter.value().expect("get value"), b"v2");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k3");
    assert_eq!(iter.value().expect("get value"), b"");

    iter.close().expect("close iter");
    state
}

fn test_delete_only_item<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k4", b"v4").expect("set k4");
    state.delete(b"k4").expect("delete k4");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state.iter(IterOptions::default()).expect("create iter");

    assert!(!iter.next().expect("yields next item"));

    iter.close().expect("close iter");
    state
}

tests!(test_delete: db + snapshot);
tests!(test_delete_last_item: db + snapshot);
tests!(test_delete_only_item: db + snapshot);
