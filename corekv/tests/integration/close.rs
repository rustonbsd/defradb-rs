use corekv::{Db, Snapshot};

use crate::{State, tests};

fn test_close<D, S>(state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.db.close();
    state
}

fn test_close_twice<D, S>(state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.db.close();
    state.db.close();
    state
}

tests!(test_close: db + snapshot);
tests!(test_close_twice: db + snapshot);
