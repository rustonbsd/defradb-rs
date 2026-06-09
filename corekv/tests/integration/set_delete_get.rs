use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_set_delete_get<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set item");
    state.delete(b"k1").expect("delete item");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    assert!(
        state
            .get(b"k1")
            .expect("returns empty value: None")
            .is_none()
    );
}

tests!(test_set_delete_get: db + snapshot);
