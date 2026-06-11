use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_set_drop_all_has<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k2", b"v2").expect("set k2");
    assert!(state.has(b"k1").expect("has k1"));
    assert!(state.has(b"k2").expect("has k2"));
    state.db.drop_all().expect("drop_all items");
    assert!(!state.has(b"k1").expect("has k1"));
    assert!(!state.has(b"k2").expect("has k2"));
    state.db.close();
    state
}

tests!(test_set_drop_all_has: db);
