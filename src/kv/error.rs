use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum KeyDecodeError {
    #[error("unknown key decode error")]
    Unknown {
        #[source]
        cause: Box<dyn std::error::Error + Send + Sync>,
    },
}