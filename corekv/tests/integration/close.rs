use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_close<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.db.close();
}

fn test_close_twice<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.db.close();
    state.db.close();
}

tests!(test_close: db + snapshot);
tests!(test_close_twice: db + snapshot);
