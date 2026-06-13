use crate::{State, tests};
use corekv::{Db, Iter, IterOptions, Snapshot};

fn test_reset_seek_next_value<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k2", b"v2").expect("set k2");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state.iter(IterOptions::default()).expect("create iter");

    iter.reset().expect("reset iter");
    assert!(iter.seek(b"k1").expect("seek key"));

    assert_eq!(iter.value().expect("get value"), b"v1");

    assert!(iter.next().expect("yield next item"));
    assert_eq!(iter.value().expect("get value"), b"v2");

    iter.close().expect("close iter");
    state
}

tests!(test_reset_seek_next_value: db + snapshot);
