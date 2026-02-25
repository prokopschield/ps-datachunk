use std::array::TryFromSliceError;

use ps_buffer::BufferError;
use ps_cypher::{DecryptionError, EncryptionError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PsDataChunkError {
    #[error(transparent)]
    BufferError(#[from] BufferError),
    #[error(transparent)]
    DecryptionError(#[from] DecryptionError),
    #[error(transparent)]
    EncryptionError(#[from] EncryptionError),
    #[error(transparent)]
    HashError(#[from] ps_hash::HashError),
    #[error(transparent)]
    HashValidationError(#[from] ps_hash::HashValidationError),
    #[error(transparent)]
    PsCypherError(#[from] ps_cypher::PsCypherError),
    #[error(transparent)]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("The data chunk was not correctly layed out")]
    InvalidDataChunk,
    #[error("The hash of a chunk was incorrect")]
    InvalidHash,
    #[error("Rkyv deserialization failed")]
    RkyvInvalidArchive,
    #[error("Rkyv serialization failed")]
    RkyvSerializationFailed,
}

pub type Result<T> = std::result::Result<T, PsDataChunkError>;
