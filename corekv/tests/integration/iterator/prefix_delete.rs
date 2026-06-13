use crate::{State, tests};
use corekv::{Db, Iter, IterOptions, Snapshot};

fn test_prefix_delete<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"1", b"v1").expect("set k1");
    state.set(b"k3", b"").expect("set k3");
    state.set(b"4", b"v4").expect("set k4");
    state.set(b"k2", b"v2").expect("set k2");
    state.delete(b"k2").expect("delete k2");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(IterOptions::builder().prefix(b"k").build())
        .expect("create iter");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k3");
    assert_eq!(iter.value().expect("get value"), b"");

    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

fn test_prefix_delete_last_item<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"1", b"v1").expect("set k1");
    state.set(b"k3", b"").expect("set k3");
    state.set(b"4", b"v4").expect("set k4");
    state.set(b"k2", b"v2").expect("set k2");
    state.delete(b"k3").expect("delete k3");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(IterOptions::builder().prefix(b"k").build())
        .expect("create iter");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k2");
    assert_eq!(iter.value().expect("get value"), b"v2");

    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

fn test_prefix_delete_only_item<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"4", b"v4").expect("set k4");
    state.delete(b"k4").expect("delete k4");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(IterOptions::builder().prefix(b"k").build())
        .expect("create iter");

    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

tests!(test_prefix_delete: db + snapshot | chunk);
tests!(test_prefix_delete_last_item: db + snapshot | chunk);
tests!(test_prefix_delete_only_item: db + snapshot);
