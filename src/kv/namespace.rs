use crate::kv::raw::RawKv;

pub struct NamespaceStore<S> {
    inner: S,
    prefix: Vec<u8>,
}

impl<S: RawKv> NamespaceStore<S> {
    pub fn new(inner: S, prefix: impl Into<Vec<u8>>) -> Self {
        Self { inner, prefix: prefix.into() }
    }

    fn qualify(&self, key: &[u8]) -> Vec<u8> {
        let mut out = self.prefix.clone();
        out.extend_from_slice(key);
        out
    }
}