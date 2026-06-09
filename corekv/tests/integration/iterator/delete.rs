use crate::{State, tests};
use corekv::{Db, Snapshot};

fn test_close_iter<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    state.set(b"k3", b"").expect("set k3");
    state.set(b"k4", b"v4").expect("set k4");
    state.set(b"k2", b"v2").expect("set k2");
    state.delete(b"k2").expect("delete k2");
}

tests!(test_close_iter: db + snapshot);
