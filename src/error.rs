use std::array::TryFromSliceError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PsDataChunkError {
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
    InvalidChecksum,
    #[error("Invalid length: {0}")]
    InvalidLength(usize),
    #[error("This should never happen: {0}")]
    ShouldNotHaveFailed(&'static str),
    #[error("DataChunk content does not match the type it is being interpreted as")]
    TypeError,
    #[error(transparent)]
    Rancor(#[from] rancor::Error),
}

pub type Result<T> = std::result::Result<T, PsDataChunkError>;
