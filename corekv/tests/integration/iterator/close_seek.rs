use corekv::{Db, Iter, IterOptions, Snapshot};

use crate::{State, get_base_error, tests};

fn test_close_seek<D, S>(mut state: State<D, S>) -> State<D, S>
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    let mut iter = state.iter(IterOptions::default()).expect("create iter");
    state.db.close();
    assert!(
        get_base_error(
            &Box::new(
                iter.seek(b"any key")
                    .expect_err("seek on closed iter errors")
            )
            .into()
        )
        .to_string()
        .ends_with("Database closed")
    );
    state
}

tests!(test_close_seek: db);
