use corekv::{Db, Iter, IterOptions, Snapshot};

use crate::{State, tests};

fn test_close_next_value<D, S>(state: &mut State<D, S>)
where
    D: Db<Snapshot = S, Iter = S::Iter>,
    S: Snapshot,
{
    state.set(b"k1", b"v1").expect("set k1");
    let mut iter = state.iter(IterOptions::default()).expect("create iter");
    iter.next().expect("yield next item");
    state.db.close();
    assert_eq!(
        iter.value()
            .expect("value on closed iter should work since snapshot lives after db close"),
        b"v1"
    );
}

tests!(test_close_next_value: db + snapshot);
