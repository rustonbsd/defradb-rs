use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_set_delete_get<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set should succeed");
    state.delete(b"k1").expect("delete should succeed");

    state
        .commit_after_writes()
        .expect("snapshot commit multiplier");
    assert!(
        state
            .get(b"k1")
            .expect("empty key should not error")
            .is_none()
    );
}

tests!(test_set_delete_get; db, snapshot);
