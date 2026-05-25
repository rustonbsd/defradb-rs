use super::KeyDecodeError;

pub trait KeyCodec: Sized {
    fn encode_key(&self) -> Vec<u8>;
    fn decode_key(raw: &[u8]) -> Result<Self, KeyDecodeError>;
}