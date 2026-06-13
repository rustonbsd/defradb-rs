use crate::{State, tests};
use corekv::{Db, Iter, IterOptions, Snapshot};

fn test_snapshot_next_value<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    // the go txn multipliers basically no-op if a NewTxn exists, we just exlude the snapshot multiplier completely
    // this assertion is just here to make that clear for anyone reading this
    assert!(matches!(state.current, crate::Active::Db(_)));

    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k3", b"v3").expect("set k3");
    state.set(b"k5", b"v5").expect("set k5");

    let mut snapshot = state
        .db
        .create_read_write_snapshot()
        .expect("create snapshot");

    snapshot.set(b"k2", b"v2").expect("set k2");
    snapshot.set(b"k4", b"v4").expect("set k4");

    let mut iter = snapshot.iter(IterOptions::default()).expect("create iter");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k1");
    assert_eq!(iter.value().expect("get value"), b"v1");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k2");
    assert_eq!(iter.value().expect("get value"), b"v2");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k3");
    assert_eq!(iter.value().expect("get value"), b"v3");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k4");
    assert_eq!(iter.value().expect("get value"), b"v4");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k5");
    assert_eq!(iter.value().expect("get value"), b"v5");

    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

fn test_snapshot_next_value_with_concurrent_addition<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    // the go txn multipliers basically no-op if a NewTxn exists, we just exlude the snapshot multiplier completely
    // this assertion is just here to make that clear for anyone reading this
    assert!(matches!(state.current, crate::Active::Db(_)));

    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k3", b"v3").expect("set k3");
    state.set(b"k5", b"v5").expect("set k5");

    let mut snapshot = state
        .db
        .create_read_write_snapshot()
        .expect("create snapshot");

    snapshot.set(b"k2", b"v2").expect("set k2");
    snapshot.set(b"k4", b"v4").expect("set k4");
    state.set(b"k4", b"v44").expect("set k4 again on db");

    let mut iter = snapshot.iter(IterOptions::default()).expect("create iter");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k1");
    assert_eq!(iter.value().expect("get value"), b"v1");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k2");
    assert_eq!(iter.value().expect("get value"), b"v2");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k3");
    assert_eq!(iter.value().expect("get value"), b"v3");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k4");
    assert_eq!(iter.value().expect("get value"), b"v4");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k5");
    assert_eq!(iter.value().expect("get value"), b"v5");

    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

fn test_snapshot_next_value_with_concurrent_update<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    // the go txn multipliers basically no-op if a NewTxn exists, we just exlude the snapshot multiplier completely
    // this assertion is just here to make that clear for anyone reading this
    assert!(matches!(state.current, crate::Active::Db(_)));

    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k3", b"v3").expect("set k3");
    state.set(b"k5", b"v5").expect("set k5");

    let mut snapshot = state
        .db
        .create_read_write_snapshot()
        .expect("create snapshot");

    snapshot.set(b"k2", b"v2").expect("set k2");
    snapshot.set(b"k4", b"v4").expect("set k4");
    state.set(b"k3", b"v33").expect("update k3 again on db");

    let mut iter = snapshot.iter(IterOptions::default()).expect("create iter");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k1");
    assert_eq!(iter.value().expect("get value"), b"v1");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k2");
    assert_eq!(iter.value().expect("get value"), b"v2");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k3");
    assert_eq!(iter.value().expect("get value"), b"v3");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k4");
    assert_eq!(iter.value().expect("get value"), b"v4");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.key().expect("get key"), b"k5");
    assert_eq!(iter.value().expect("get value"), b"v5");

    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

tests!(test_snapshot_next_value: db);
tests!(test_snapshot_next_value_with_concurrent_addition: db);
tests!(test_snapshot_next_value_with_concurrent_update: db);
