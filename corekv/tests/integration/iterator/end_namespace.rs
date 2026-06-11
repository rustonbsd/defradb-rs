use corekv::{Db, Iter, IterOptions, PrefixKey, Snapshot};

use crate::{State, tests};

fn test_end_namespace_exclude_items_outside_of_namespace<D, S>(
    mut state: State<D, S>,
) -> State<PrefixKey<D>, PrefixKey<S>>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k5", b"v5").expect("set k5");

    let mut state = state.wrap_prefix_key(b"namespace");

    state.set(b"k2", b"v2").expect("set k2");
    state.set(b"k3", b"v3").expect("set k3");
    state.set(b"k4", b"v4").expect("set k4");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(IterOptions::builder().key_range_end(b"k4").build())
        .expect("create iter");
    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k2");
    assert_eq!(iter.value().expect("get value"), b"v2");
    iter.next().expect("yield next item");
    assert_eq!(iter.key().expect("get key"), b"k3");
    assert_eq!(iter.value().expect("get value"), b"v3");
    iter.next().expect("yield next item");
    assert!(!iter.next().expect("yields no next item"));
    iter.close().expect("close iter");

    state
}

tests!(test_end_namespace_exclude_items_outside_of_namespace: db + snapshot | chunk);
