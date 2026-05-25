pub trait RawKv {
    type Error;
    type Iter<'a>: KvIter<Error = Self::Error>
    where
        Self: 'a;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;
    fn has(&self, key: &[u8]) -> Result<bool, Self::Error>;
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;
    fn delete(&mut self, key: &[u8]) -> Result<(), Self::Error>;

    fn iter(&self, opts: IterOptions<'_>) -> Result<Self::Iter<'_>, Self::Error>;
}

pub trait KvIter {
    type Error;
    fn next(&mut self) -> Result<bool, Self::Error>;
    fn key(&self) -> &[u8];
    fn value(&self) -> Result<&[u8], Self::Error>;
}

#[derive(Clone, Debug, Default)]
pub struct IterOptions<'a> {
    pub prefix: Option<&'a [u8]>,
    pub start: Option<&'a [u8]>,
    pub end: Option<&'a [u8]>,
    pub keys_only: bool,
    pub reverse: bool,
}
