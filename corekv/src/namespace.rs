use crate::{
    Db, Iter, IterOptions, NewIter, Reader, Snapshot, SnapshotCreator, Writer, traits::ErrorFamily,
};

#[derive(Debug, thiserror::Error)]
pub enum PrefixKeyAccessError<E>
where
    E: std::error::Error + 'static,
{
    #[error("prefix key empty key")]
    EmptyKey,

    #[error("prefix key get failed")]
    Get(#[source] E),

    #[error("prefix key has failed")]
    Has(#[source] E),

    #[error("prefix key set failed")]
    Set(#[source] E),

    #[error("prefix key delete failed")]
    Delete(#[source] E),

    #[error("prefix key iter creation failed")]
    IterCreate(#[source] E),

    #[error("prefix key snapshot creation failed")]
    CreateSnapshot(#[source] E),

    #[error("prefix key drop_all failed")]
    DropAll(#[source] E),
}

#[derive(Debug, thiserror::Error)]
pub enum PrefixKeySnapshotError<E>
where
    E: std::error::Error + 'static,
{
    #[error("prefix key commit failed")]
    Commit(#[source] E),
}

#[derive(Debug, thiserror::Error)]
pub enum PrefixKeyIterError<E>
where
    E: std::error::Error + 'static,
{
    #[error("prefix key iterator returned key outside prefix")]
    InvalidPrefix,

    #[error("prefix key iter next failed")]
    Next(#[source] E),

    #[error("prefix key iter key failed")]
    Key(#[source] E),

    #[error("prefix key iter value failed")]
    Value(#[source] E),

    #[error("prefix key iter seek failed")]
    Seek(#[source] E),

    #[error("prefix key iter reset failed")]
    Reset(#[source] E),

    #[error("prefix key iter close failed")]
    Close(#[source] E),
}

impl<T> ErrorFamily for PrefixKey<T>
where
    T: ErrorFamily,
{
    type AccessError = PrefixKeyAccessError<T::AccessError>;
    type IterError = PrefixKeyIterError<T::IterError>;
    type SnapshotError = PrefixKeySnapshotError<T::SnapshotError>;
}

#[derive(Debug, Clone)]
pub struct PrefixKey<T> {
    inner: T,
    prefix: Vec<u8>,
}

#[derive(Debug)]
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
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::AccessError> {
        self.inner
            .get(&prefix_key(&self.prefix, key))
            .map_err(PrefixKeyAccessError::Get)
    }

    fn has(&self, key: &[u8]) -> Result<bool, Self::AccessError> {
        self.inner
            .has(&prefix_key(&self.prefix, key))
            .map_err(PrefixKeyAccessError::Has)
    }
}

impl<T> Writer for PrefixKey<T>
where
    T: Writer,
{
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::AccessError> {
        if key.is_empty() {
            return Err(PrefixKeyAccessError::EmptyKey);
        }
        self.inner
            .set(&prefix_key(&self.prefix, key), value)
            .map_err(PrefixKeyAccessError::Set)
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), Self::AccessError> {
        if key.is_empty() {
            return Err(PrefixKeyAccessError::EmptyKey);
        }
        self.inner
            .delete(&prefix_key(&self.prefix, key))
            .map_err(PrefixKeyAccessError::Delete)
    }
}

impl<T> NewIter for PrefixKey<T>
where
    T: NewIter,
{
    type Iter = PrefixKeyIter<T::Iter>;

    // ref: /corekv/namespace/namespace.go -> func (nstore *Datastore) Iterator(ctx context.Context, opts corekv.IterOptions)
    fn iter(&self, opts: IterOptions) -> Result<Self::Iter, Self::AccessError> {
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
            .map_err(PrefixKeyAccessError::IterCreate)?;

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
    type IterError = PrefixKeyIterError<T::IterError>;

    fn next(&mut self) -> Result<bool, Self::IterError> {
        self.inner
            .next()
            .map_err(PrefixKeyIterError::Next)
    }

    fn key(&self) -> Result<Vec<u8>, Self::IterError> {
        let key = self
            .inner
            .key()
            .map_err(PrefixKeyIterError::Key)?;
        key.strip_prefix(self.prefix.as_slice())
            .map(|k| k.to_vec())
            .ok_or_else(|| PrefixKeyIterError::InvalidPrefix)
    }

    fn value(&self) -> Result<Vec<u8>, Self::IterError> {
        self.inner
            .value()
            .map_err(PrefixKeyIterError::Value)
    }

    fn seek(&mut self, key: &[u8]) -> Result<bool, Self::IterError> {
        self.inner
            .seek(&prefix_key(&self.prefix, key))
            .map_err(PrefixKeyIterError::Seek)
    }

    fn reset(&mut self) -> Result<(), Self::IterError> {
        self.inner
            .reset()
            .map_err(PrefixKeyIterError::Reset)
    }

    fn close(&mut self) -> Result<(), Self::IterError> {
        self.inner
            .close()
            .map_err(PrefixKeyIterError::Close)
    }
}

impl<T> Db for PrefixKey<T>
where
    T: Db,
{
    fn close(&self) {
        self.inner.close()
    }

    fn drop_all(&self) -> Result<(), Self::AccessError> {
        self.inner
            .drop_all()
            .map_err(PrefixKeyAccessError::DropAll)
    }

    fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

impl<T> SnapshotCreator for PrefixKey<T>
where
    T: SnapshotCreator,
{
    type Snapshot = PrefixKey<T::Snapshot>;

    fn create_read_only_snapshot(&self) -> Result<Self::Snapshot, Self::AccessError> {
        let snapshot = self
            .inner
            .create_read_only_snapshot()
            .map_err(PrefixKeyAccessError::CreateSnapshot)?;
        Ok(PrefixKey::wrap(snapshot, self.prefix.clone()))
    }

    fn create_read_write_snapshot(&self) -> Result<Self::Snapshot, Self::AccessError> {
        let snapshot = self
            .inner
            .create_read_write_snapshot()
            .map_err(PrefixKeyAccessError::CreateSnapshot)?;
        Ok(PrefixKey::wrap(snapshot, self.prefix.clone()))
    }
}

impl<T> Snapshot for PrefixKey<T>
where
    T: Snapshot,
{
    fn commit(&self) -> Result<(), Self::SnapshotError> {
        self.inner
            .commit()
            .map_err(PrefixKeySnapshotError::Commit)
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
