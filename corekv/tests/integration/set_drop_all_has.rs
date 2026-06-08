use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_set_drop_all_has<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set should succeed");
    state.set(b"k2", b"v2").expect("set should succeed");
    assert!(state.has(b"k1").expect("has should succeed"));
    assert!(state.has(b"k2").expect("has should succeed"));
    state.db.drop_all().expect("drop_all should succeed");
    assert!(!state.has(b"k1").expect("has should succeed"));
    assert!(!state.has(b"k2").expect("has should succeed"));
    state.db.close()
}

tests!(test_set_drop_all_has; db);
