use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering::Relaxed},
};

use crate::{
    Db, Iter, IterOptions, NewIter, Reader, Snapshot, SnapshotCreator, Writer, traits::ErrorFamily,
};

#[derive(Debug, thiserror::Error)]
pub enum ChunkAccessError<AE, IE>
where
    AE: std::error::Error + 'static,
    IE: std::error::Error + 'static,
{
    #[error("chunk empty key")]
    EmptyKey,

    #[error("chunk get failed while creating iterator")]
    GetCreateIter(#[source] AE),

    #[error("chunk get failed while reading iterator")]
    GetIter(#[source] IE),

    #[error("chunk has failed while creating iterator")]
    HasCreateIter(#[source] AE),

    #[error("chunk has failed while reading iterator")]
    HasIter(#[source] IE),

    #[error("chunk set failed")]
    Set(#[source] AE),

    #[error("chunk delete failed while creating iterator")]
    DeleteCreateIter(#[source] AE),

    #[error("chunk delete failed while reading iterator")]
    DeleteIter(#[source] IE),

    #[error("chunk delete failed while deleting inner chunk")]
    DeleteInner(#[source] AE),

    #[error("chunk iter creation failed")]
    IterCreate(#[source] AE),

    #[error("chunk snapshot creation failed")]
    CreateSnapshot(#[source] AE),

    #[error("chunk drop_all failed")]
    DropAll(#[source] AE),
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkSnapshotError<SE>
where
    SE: std::error::Error + 'static,
{
    #[error("chunk commit failed")]
    Commit(#[source] SE),
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkIterError<IE>
where
    IE: std::error::Error + 'static,
{
    #[error("chunk iter next failed")]
    Next(#[source] IE),

    #[error("chunk iter failed reading chunk key")]
    ReadChunkKey(#[source] IE),

    #[error("chunk iter failed reading chunk value")]
    ReadChunkValue(#[source] IE),

    #[error("chunk iter seek failed")]
    Seek(#[source] IE),

    #[error("chunk iter reset failed")]
    Reset(#[source] IE),

    #[error("chunk iter close failed")]
    Close(#[source] IE),
}

impl<T> ErrorFamily for Chunk<T>
where
    T: ErrorFamily,
{
    type AccessError = ChunkAccessError<T::AccessError, T::IterError>;
    type IterError = ChunkIterError<T::IterError>;
    type SnapshotError = ChunkSnapshotError<T::SnapshotError>;
}

#[derive(Debug, Clone)]
pub struct Chunk<T> {
    inner: T,
    chunk_size: usize,
    key_len: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub struct ChunkIter<I> {
    inner: I,
    key_len: usize,
    opt_reverse: bool,

    current_key: Vec<u8>,
    current_value: Vec<u8>,
    current_chunk: Vec<u8>,
    current_chunk_key: Vec<u8>,
}

impl<T> Chunk<T>
where
    T: NewIter,
{
    pub fn wrap(inner: T, chunk_size: usize, key_len: Option<usize>) -> Self {
        let key_len = match key_len {
            Some(key_len) => Some(key_len),
            None => Self::try_identify_key_len(&inner),
        };

        Self {
            inner,
            chunk_size,
            key_len: Arc::new(AtomicUsize::new(key_len.unwrap_or_default())),
        }
    }

    fn try_identify_key_len(inner: &T) -> Option<usize> {
        if let Ok(mut iter) = inner.iter(IterOptions::builder().keys_only(true).build()) {
            let key_len = if iter.next().is_ok()
                && let Ok(key) = iter.key()
            {
                Some(key.len() - 1)
            } else {
                None
            };
            let _ = iter.close();
            return key_len;
        }
        None
    }
}

impl<T> Reader for Chunk<T>
where
    T: Reader + NewIter,
{
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::AccessError> {
        let mut iter = self
            .inner
            .iter(IterOptions::builder().prefix(key).build())
            .map_err(ChunkAccessError::GetCreateIter)?;
        let mut chunks = Vec::new();
        while iter.next().map_err(ChunkAccessError::GetIter)? {
            let chunk = iter.value().map_err(ChunkAccessError::GetIter)?;
            chunks.push(chunk);
        }

        iter.close().map_err(ChunkAccessError::GetIter)?;

        if chunks.is_empty() {
            return Ok(None);
        }
        Ok(Some(chunks.concat()))
    }

    fn has(&self, key: &[u8]) -> Result<bool, Self::AccessError> {
        let mut iter = self
            .inner
            .iter(IterOptions::builder().prefix(key).keys_only(true).build())
            .map_err(ChunkAccessError::HasCreateIter)?;
        let has_next = iter.next().map_err(ChunkAccessError::HasIter)?;
        iter.close().map_err(ChunkAccessError::HasIter)?;
        Ok(has_next)
    }
}

impl<T> Writer for Chunk<T>
where
    T: Writer + NewIter,
{
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::AccessError> {
        let mut chunk_key = [key, &[0]].concat();
        if value.is_empty() {
            self.inner
                .set(&chunk_key, &[])
                .map_err(ChunkAccessError::Set)?;
            return Ok(());
        }

        for chunk in value.chunks(self.chunk_size) {
            self.inner
                .set(&chunk_key, chunk)
                .map_err(ChunkAccessError::Set)?;
            bytes_prefix_end(&mut chunk_key);
        }

        // If this is the first write, set the length of keys
        if self.key_len.load(Relaxed) == 0 {
            self.key_len.store(key.len(), Relaxed);
        }
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> Result<(), Self::AccessError> {
        let mut iter = self
            .inner
            .iter(IterOptions::builder().prefix(key).keys_only(true).build())
            .map_err(ChunkAccessError::DeleteCreateIter)?;
        let mut chunk_keys = Vec::new();
        while iter.next().map_err(ChunkAccessError::DeleteIter)? {
            let chunk_key = iter.key().map_err(ChunkAccessError::DeleteIter)?;
            chunk_keys.push(chunk_key);
        }
        iter.close().map_err(ChunkAccessError::DeleteIter)?;
        for chunk_key in chunk_keys {
            self.inner
                .delete(&chunk_key)
                .map_err(ChunkAccessError::DeleteInner)?;
        }
        Ok(())
    }
}

fn bytes_prefix_end(key: &mut [u8]) {
    for i in (0..key.len()).rev() {
        key[i] = key[i].wrapping_add(1);
        if key[i] != 0 {
            return;
        }
    }
}

impl<T> NewIter for Chunk<T>
where
    T: NewIter,
{
    type Iter = ChunkIter<T::Iter>;

    fn iter(&self, opts: crate::IterOptions) -> Result<Self::Iter, Self::AccessError> {
        if self.key_len.load(Relaxed) == 0
            && let Some(key_len) = Self::try_identify_key_len(&self.inner)
        {
            self.key_len.store(key_len, Relaxed);
        }
        let iter = self
            .inner
            .iter(opts.clone())
            .map_err(ChunkAccessError::IterCreate)?;
        Ok(ChunkIter {
            inner: iter,
            key_len: self.key_len.load(Relaxed),
            opt_reverse: opts.reverse(),
            current_key: Vec::new(),
            current_value: Vec::new(),
            current_chunk: Vec::new(),
            current_chunk_key: Vec::new(),
        })
    }
}

impl<T> Iter for ChunkIter<T>
where
    T: Iter,
{
    type IterError = ChunkIterError<T::IterError>;

    fn next(&mut self) -> Result<bool, Self::IterError> {
        let mut chunks = Vec::new();
        let mut empty_chunk = false;
        loop {
            if !self.current_chunk_key.is_empty() {
                if self.opt_reverse {
                    chunks.insert(0, self.current_chunk.clone());
                } else {
                    chunks.push(self.current_chunk.clone());
                }
                self.current_key = self.current_chunk_key[..self.key_len].to_vec();
                self.current_chunk_key.clear();
                self.current_chunk.clear();
            }

            if !self.inner.next().map_err(ChunkIterError::Next)? {
                break;
            }

            self.current_chunk_key = self.inner.key().map_err(ChunkIterError::Next)?;
            self.current_chunk = self.inner.value().map_err(ChunkIterError::Next)?;

            if !self.current_chunk_key.starts_with(&self.current_key) {
                if self.current_chunk.is_empty() {
                    empty_chunk = true;
                }
                break;
            }
        }
        if chunks.is_empty() && !empty_chunk {
            return Ok(false);
        }
        self.current_value = chunks.concat();
        Ok(true)
    }

    fn key(&self) -> Result<Vec<u8>, Self::IterError> {
        Ok(self.current_key.clone())
    }

    fn value(&self) -> Result<Vec<u8>, Self::IterError> {
        Ok(self.current_value.clone())
    }

    fn seek(&mut self, key: &[u8]) -> Result<bool, Self::IterError> {
        let key = if self.opt_reverse {
            let mut key = key.to_vec();
            bytes_prefix_end(&mut key);
            key
        } else {
            key.to_vec()
        };
        if !self.inner.seek(&key).map_err(ChunkIterError::Seek)? {
            return Ok(false);
        }
        self.current_chunk_key = self.inner.key().map_err(ChunkIterError::Seek)?;
        self.current_chunk = self.inner.value().map_err(ChunkIterError::Seek)?;
        self.next()
    }

    fn reset(&mut self) -> Result<(), Self::IterError> {
        self.current_key.clear();
        self.current_value.clear();
        self.current_chunk.clear();
        self.current_chunk_key.clear();
        self.inner.reset().map_err(ChunkIterError::Reset)
    }

    fn close(&mut self) -> Result<(), Self::IterError> {
        self.inner.close().map_err(ChunkIterError::Close)
    }
}

impl<T> Db for Chunk<T>
where
    T: Db,
{
    fn close(&self) {
        self.inner.close()
    }

    fn drop_all(&self) -> Result<(), Self::AccessError> {
        self.inner.drop_all().map_err(ChunkAccessError::DropAll)
    }

    fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }
}

// todo: might need a custom impl to allow db independent chunk_size
impl<T> SnapshotCreator for Chunk<T>
where
    T: SnapshotCreator,
{
    type Snapshot = Chunk<T::Snapshot>;

    fn create_read_only_snapshot(&self) -> Result<Self::Snapshot, Self::AccessError> {
        let snapshot = self
            .inner
            .create_read_only_snapshot()
            .map_err(ChunkAccessError::CreateSnapshot)?;
        Ok(Chunk::wrap(snapshot, self.chunk_size, None))
    }

    fn create_read_write_snapshot(&self) -> Result<Self::Snapshot, Self::AccessError> {
        let snapshot = self
            .inner
            .create_read_write_snapshot()
            .map_err(ChunkAccessError::CreateSnapshot)?;
        Ok(Chunk::wrap(snapshot, self.chunk_size, None))
    }
}

impl<T> Snapshot for Chunk<T>
where
    T: Snapshot,
{
    fn commit(&self) -> Result<(), Self::SnapshotError> {
        self.inner.commit().map_err(ChunkSnapshotError::Commit)
    }

    fn discard(&self) {
        self.inner.discard()
    }
}
