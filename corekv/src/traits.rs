pub trait Reader {
    type Error;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn has(&self, key: &[u8]) -> Result<bool, Self::Error>;
}

pub trait Writer {
    type Error;

    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&mut self, key: &[u8]) -> Result<(), Self::Error>;
}

pub trait ReaderWriterIter {
    type IterError;
    type Iter;
    fn iter(&self, opts: IterOptions) -> Result<Self::Iter, Self::IterError>;
}

#[derive(Clone, Debug, Default)]
pub struct IterOptions(badger_rs::IteratorOptions);

pub struct IterOptionsBuilder(badger_rs::IteratorOptions);
impl IterOptionsBuilder {
    pub fn prefix(mut self, prefix: &[u8]) -> Self {
        self.0.prefix = prefix.to_vec();
        self
    }

    pub fn key_range_start(mut self, start: &[u8]) -> Self {
        self.0.start = start.to_vec();
        self
    }

    pub fn key_range_end(mut self, end: &[u8]) -> Self {
        self.0.end = end.to_vec();
        self
    }

    pub fn reverse(mut self, reverse: bool) -> Self {
        self.0.reverse = reverse;
        self
    }

    pub fn keys_only(mut self, keys_only: bool) -> Self {
        self.0.keys_only = keys_only;
        self
    }

    pub fn build(self) -> IterOptions {
        IterOptions(self.0)
    }
}

impl IterOptions {
    pub fn builder() -> IterOptionsBuilder {
        IterOptionsBuilder(badger_rs::IteratorOptions::default())
    }

    pub fn builder_from(opts: &Self) -> IterOptionsBuilder {
        IterOptionsBuilder(opts.0.clone())
    }

    pub fn prefix(&self) -> &[u8] {
        &self.0.prefix
    }

    pub fn key_range_start(&self) -> &[u8] {
        &self.0.start
    }

    pub fn key_range_end(&self) -> &[u8] {
        &self.0.end
    }

    pub fn reverse(&self) -> bool {
        self.0.reverse
    }

    pub fn keys_only(&self) -> bool {
        self.0.keys_only
    }
}

impl From<IterOptions> for badger_rs::IteratorOptions {
    fn from(opts: IterOptions) -> Self {
        opts.0
    }
}

pub trait Iter {
    type IterError;
    fn next(&mut self) -> Result<bool, Self::IterError>;
    fn key(&self) -> Result<Vec<u8>, Self::IterError>;
    fn value(&self) -> Result<Vec<u8>, Self::IterError>;
    fn seek(&mut self, key: &[u8]) -> Result<bool, Self::IterError>;
    fn reset(&mut self) -> Result<(), Self::IterError>;
    fn close(&mut self) -> Result<(), Self::IterError>;
}

pub trait ReaderWriterIterType: Reader + Writer + ReaderWriterIter<Iter: Iter> {}

pub trait SnapshotCreator {
    type Snapshot: Snapshot;
    type Error;
    fn create_read_only_snapshot(&self) -> Result<Self::Snapshot, Self::Error>;
    fn create_read_write_snapshot(&self) -> Result<Self::Snapshot, Self::Error>;
}

pub trait Snapshot: ReaderWriterIterType + Clone + Sync {
    type SnapshotError;
    fn commit(&self) -> Result<(), Self::SnapshotError>;
    fn discard(&self);
}

pub trait Db: ReaderWriterIterType + SnapshotCreator + Clone + Sync {
    type DbError;
    fn close(&self);
    fn drop_all(&self) -> Result<(), Self::DbError>;
}
