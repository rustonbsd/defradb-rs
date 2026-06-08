use std::{
    error::Error,
    fmt::{Debug, Display},
};

pub trait ErrorFamily {
    type AccessError: Error + Display + Debug + Send + Sync + 'static;
    type IterError: Error + Display + Debug + Send + Sync + 'static;
    type SnapshotError: Error + Display + Debug + Send + Sync + 'static;
}

pub trait Reader: ErrorFamily {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::AccessError>;
    fn has(&self, key: &[u8]) -> Result<bool, Self::AccessError>;
}


pub trait Writer: ErrorFamily {
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::AccessError>;
    fn delete(&mut self, key: &[u8]) -> Result<(), Self::AccessError>;
}

pub trait NewIter: ErrorFamily {
    type Iter: Iter<IterError = Self::IterError>;
    fn iter(&self, opts: IterOptions) -> Result<Self::Iter, Self::AccessError>;
}

pub trait Iter {
    type IterError: Error + Display + Debug + Send + Sync + 'static;
    fn next(&mut self) -> Result<bool, Self::IterError>;
    fn key(&self) -> Result<Vec<u8>, Self::IterError>;
    fn value(&self) -> Result<Vec<u8>, Self::IterError>;
    fn seek(&mut self, key: &[u8]) -> Result<bool, Self::IterError>;
    fn reset(&mut self) -> Result<(), Self::IterError>;
    fn close(&mut self) -> Result<(), Self::IterError>;
}

pub trait SnapshotCreator: ErrorFamily {
    type Snapshot: Snapshot<SnapshotError = Self::SnapshotError>;
    fn create_read_only_snapshot(&self) -> Result<Self::Snapshot, Self::AccessError>;
    fn create_read_write_snapshot(&self) -> Result<Self::Snapshot, Self::AccessError>;
}

pub trait Snapshot: Reader + Writer + NewIter + Clone + Sync + ErrorFamily {
    fn commit(&self) -> Result<(), Self::SnapshotError>;
    fn discard(&self);
}

pub trait Db: Reader + Writer + NewIter + SnapshotCreator + Clone + Sync + ErrorFamily {
    fn close(&self);
    fn drop_all(&self) -> Result<(), Self::AccessError>;
    fn is_closed(&self) -> bool;
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
