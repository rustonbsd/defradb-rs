use corekv::{Db, Snapshot};

use crate::{State, get_base_error, tests};

fn test_close_then_set<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.db.close();
    assert!(
        get_base_error(
            &state
                .set(b"not important", b"value")
                .expect_err("should error")
        )
        .to_string()
        .ends_with("db is closed")
    );
    state
}

tests!(test_close_then_set: db);
