use std::array::TryFromSliceError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataChunkError {
    #[error(transparent)]
    Buffer(#[from] ps_buffer::BufferError),
    #[error(transparent)]
    Decryption(#[from] ps_cypher::DecryptionError),
    #[error(transparent)]
    Encryption(#[from] ps_cypher::EncryptionError),
    #[error(transparent)]
    Hash(#[from] ps_hash::HashError),
    #[error(transparent)]
    HashValidation(#[from] ps_hash::HashValidationError),
    #[error(transparent)]
    Slice(#[from] TryFromSliceError),
    #[error("The data chunk was not correctly layed out")]
    InvalidLayout,
    #[error("The hash of a chunk was incorrect")]
    HashMismatch,
    #[error("Rkyv deserialization failed")]
    InvalidArchive,
    #[error("Rkyv serialization failed")]
    Serialization,
}

pub type Result<T> = std::result::Result<T, DataChunkError>;
