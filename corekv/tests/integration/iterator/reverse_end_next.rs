use crate::{State, tests};
use corekv::{Db, Iter, IterOptions, Snapshot};

fn test_reverse_end_next<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");

    let mut iter = state
        .iter(
            IterOptions::builder()
                .reverse(true)
                .key_range_end(b"k4")
                .build(),
        )
        .expect("create iter");

    assert!(!iter.next().expect("yields no next item"));

    iter.close().expect("close iter");
    state
}

tests!(test_reverse_end_next: db + snapshot);
