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
    PsHashError(#[from] ps_hash::PsHashError),
    #[error(transparent)]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("Failed to serialize into an AlignedDataChunk")]
    SerializationError,
    #[error("The data chunk was not correctly layed out")]
    InvalidDataChunk,
    #[error("The hash of a chunk was incorrect")]
    InvalidHash,
    #[error("Invalid length: {0}")]
    InvalidLength(usize),
    #[error(transparent)]
    RkyvInvalidArchive(anyhow::Error),
    #[error(transparent)]
    RkyvSerializationFailed(anyhow::Error),
    #[error("This should never happen: {0}")]
    ShouldNotHaveFailed(&'static str),
    #[error("DataChunk content does not match the type it is being interpreted as")]
    TypeError,
}

pub type Result<T> = std::result::Result<T, PsDataChunkError>;
