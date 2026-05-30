// ref: /keyring/keyring.go

pub const KEYRING_BACKEND_FILE: &str = "file";
// pub const KEYRING_BACKEND_SYSTEM: &str = "system"; // not yet implemented

pub trait KeyringT {
    type Error;

    fn set(&mut self, name: &str, key: &[u8]) -> Result<(), Self::Error>;
    fn get(&self, name: &str) -> Result<Vec<u8>, Self::Error>;
    fn delete(&mut self, name: &str) -> Result<(), Self::Error>;
    fn list(&self) -> Result<Vec<String>, Self::Error>;
}