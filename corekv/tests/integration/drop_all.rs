use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_drop_all<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    assert!(state.drop_all().is_ok());
    state
}

tests!(test_drop_all: db);
