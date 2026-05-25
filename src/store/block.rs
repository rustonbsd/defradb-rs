use ipld_core::cid::Cid;

use crate::kv::RawKv;

pub struct BlockStore<S> {
    raw: S,
    rehash: bool,
}