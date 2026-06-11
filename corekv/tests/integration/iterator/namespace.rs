use corekv::{Db, Iter, IterOptions, PrefixKey, Snapshot};

use crate::{State, tests};

fn test_namespace_does_not_return_self_namespace_match<D, S>(
    mut state: State<D, S>,
) -> State<PrefixKey<D>, PrefixKey<S>>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state
        .set(b"namespace", b"namespace exact match")
        .expect("set namespace");

    let mut state = state.wrap_prefix_key(b"namespace");

    state.set(b"k1", b"v1").expect("set k1");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state.iter(IterOptions::default()).expect("create iter");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"");
    assert_eq!(iter.value().expect("get value"), b"namespace exact match");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k1");
    assert_eq!(iter.value().expect("get value"), b"v1");
    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");

    state
}

fn test_namespace_excludes_items_outside_of_namespace<D, S>(
    mut state: State<D, S>,
) -> State<PrefixKey<D>, PrefixKey<S>>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k4", b"v4").expect("set k4");

    let mut state = state.wrap_prefix_key(b"namespace");

    state.set(b"k2", b"v2").expect("set k2");
    state.set(b"k3", b"v3").expect("set k3");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state.iter(IterOptions::default()).expect("create iter");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k2");
    assert_eq!(iter.value().expect("get value"), b"v2");

    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k3");
    assert_eq!(iter.value().expect("get value"), b"v3");
    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");

    state
}

tests!(test_namespace_does_not_return_self_namespace_match: db + snapshot | chunk);
tests!(test_namespace_excludes_items_outside_of_namespace: db + snapshot | chunk);
