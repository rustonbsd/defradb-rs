use crate::{State, tests};
use corekv::{Db, Iter, IterOptions, Snapshot};

fn test_prefix_reverse_next<D, S>(
    mut state: State<D, S>,
) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(IterOptions::builder().reverse(true).prefix(b"k").build())
        .expect("create iter");

    assert!(iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

tests!(test_prefix_reverse_next: db + snapshot);
