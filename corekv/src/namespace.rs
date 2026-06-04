use crate::{
    Db, Iter, IterOptions, Reader, ReaderWriterIter, ReaderWriterIterType, Snapshot,
    SnapshotCreator, Writer,
};

#[derive(Debug, thiserror::Error)]
pub enum PrefixKeyError<E> {
    #[error("key cannot be empty")]
    EmptyKey,
    #[error("inner error: {0}")]
    InnerError(#[source] E),
    #[error("invalid prefix in iterator")]
    InvalidPrefix,
}

#[derive(Debug, Clone)]
pub struct PrefixKey<T> {
    inner: T,
    prefix: Vec<u8>,
}

pub struct PrefixKeyIter<I> {
    inner: I,
    prefix: Vec<u8>,
}

impl<T> PrefixKey<T> {
    pub fn wrap(inner: T, prefix: Vec<u8>) -> Self {
        Self { inner, prefix }
    }
}

impl<T> Reader for PrefixKey<T>
where
    T: Reader,
{
    type Error = PrefixKeyError<T::Error>;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        self.inner
            .get(&prefix_key(&self.prefix, key))
            .map_err(Self::Error::InnerError)
    }

    fn has(&self, key: &[u8]) -> Result<bool, Self::Error> {
        self.inner
            .has(&prefix_key(&self.prefix, key))
            .map_err(Self::Error::InnerError)
    }
}

impl<T> Writer for PrefixKey<T>
where
    T: Writer,
{
    type Error = PrefixKeyError<T::Error>;

    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        if key.is_empty() {
            return Err(Self::Error::EmptyKey);
        }
        self.inner
            .set(&prefix_key(&self.prefix, key), value)
            .map_err(Self::Error::InnerError)
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), Self::Error> {
        if key.is_empty() {
            return Err(Self::Error::EmptyKey);
        }
        self.inner
            .delete(&prefix_key(&self.prefix, key))
            .map_err(Self::Error::InnerError)
    }
}

impl<T> ReaderWriterIter for PrefixKey<T>
where
    T: ReaderWriterIter,
{
    type IterError = PrefixKeyError<T::IterError>;
    type Iter = PrefixKeyIter<T::Iter>;

    // ref: /corekv/namespace/namespace.go -> func (nstore *Datastore) Iterator(ctx context.Context, opts corekv.IterOptions)
    fn iter(&self, opts: IterOptions) -> Result<Self::Iter, Self::IterError> {
        let mut p_opts = IterOptions::builder_from(&opts);
        if !opts.prefix().is_empty() {
            p_opts = p_opts.prefix(&prefix_key(&self.prefix, opts.prefix()));
        } else if !opts.key_range_start().is_empty() || !opts.key_range_end().is_empty() {
            p_opts = p_opts.key_range_start(&prefix_key(&self.prefix, opts.key_range_start()));
            let range_end_key = if !opts.key_range_end().is_empty() {
                prefix_key(&self.prefix, opts.key_range_end())
            } else {
                let mut range_end_key = self.prefix.clone();
                for i in (0..range_end_key.len()).rev() {
                    range_end_key[i] = range_end_key[i].wrapping_add(1);
                    if range_end_key[i] != 0 {
                        range_end_key.truncate(i + 1);
                        break;
                    }
                }
                range_end_key
            };
            p_opts = p_opts.key_range_end(&range_end_key);
        } else {
            // opts.prefix() is empty so adding the prefix to it will just be the prefix itself
            p_opts = p_opts.prefix(&self.prefix);
        }

        let inner_iter = self
            .inner
            .iter(p_opts.build())
            .map_err(Self::IterError::InnerError)?;

        Ok(PrefixKeyIter {
            inner: inner_iter,
            prefix: self.prefix.clone(),
        })
    }
}

impl<T> Iter for PrefixKeyIter<T>
where
    T: Iter,
{
    type IterError = PrefixKeyError<T::IterError>;

    fn next(&mut self) -> Result<bool, Self::IterError> {
        self.inner.next().map_err(Self::IterError::InnerError)
    }

    fn key(&self) -> Result<Vec<u8>, Self::IterError> {
        let key = self.inner.key().map_err(Self::IterError::InnerError)?;
        key.strip_prefix(self.prefix.as_slice())
            .map(|k| k.to_vec())
            .ok_or_else(|| Self::IterError::InvalidPrefix)
    }

    fn value(&self) -> Result<Vec<u8>, Self::IterError> {
        self.inner.value().map_err(Self::IterError::InnerError)
    }

    fn seek(&mut self, key: &[u8]) -> Result<bool, Self::IterError> {
        self.inner
            .seek(&prefix_key(&self.prefix, key))
            .map_err(Self::IterError::InnerError)
    }

    fn reset(&mut self) -> Result<(), Self::IterError> {
        self.inner.reset().map_err(Self::IterError::InnerError)
    }

    fn close(&mut self) -> Result<(), Self::IterError> {
        self.inner.close().map_err(Self::IterError::InnerError)
    }
}

impl<T> ReaderWriterIterType for PrefixKey<T> where T: ReaderWriterIterType {}
impl<T> Db for PrefixKey<T>
where
    T: Db,
{
    type DbError = PrefixKeyError<T::DbError>;

    fn close(&self) {
        self.inner.close()
    }

    fn drop_all(&self) -> Result<(), Self::DbError> {
        self.inner.drop_all().map_err(Self::DbError::InnerError)
    }
}

impl<T> SnapshotCreator for PrefixKey<T>
where
    T: SnapshotCreator,
{
    type Snapshot = PrefixKey<T::Snapshot>;
    type Error = PrefixKeyError<T::Error>;

    fn create_read_only_snapshot(&self) -> Result<Self::Snapshot, Self::Error> {
        let snapshot = self
            .inner
            .create_read_only_snapshot()
            .map_err(Self::Error::InnerError)?;
        Ok(PrefixKey::wrap(snapshot, self.prefix.clone()))
    }

    fn create_read_write_snapshot(&self) -> Result<Self::Snapshot, Self::Error> {
        let snapshot = self
            .inner
            .create_read_write_snapshot()
            .map_err(Self::Error::InnerError)?;
        Ok(PrefixKey::wrap(snapshot, self.prefix.clone()))
    }
}

impl<T> Snapshot for PrefixKey<T>
where
    T: Snapshot,
{
    type SnapshotError = PrefixKeyError<T::SnapshotError>;

    fn commit(&self) -> Result<(), Self::SnapshotError> {
        self.inner.commit().map_err(Self::SnapshotError::InnerError)
    }

    fn discard(&self) {
        self.inner.discard()
    }
}

fn prefix_key(prefix: &[u8], key: &[u8]) -> Vec<u8> {
    let mut out = prefix.to_vec();
    out.extend_from_slice(key);
    out
}
