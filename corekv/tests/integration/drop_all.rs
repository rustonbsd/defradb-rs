use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_drop_all<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    assert!(state.drop_all().is_ok());
}

tests!(test_drop_all; db);
