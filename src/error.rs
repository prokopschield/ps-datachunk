use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PsDataChunkError {
    #[error(transparent)]
    PsCypherError(#[from] ps_cypher::PsCypherError),
    #[error("Failed to serialize into an AlignedDataChunk")]
    SerializationError,
    #[error("The data chunk was not correctly layed out")]
    InvalidDataChunk,
    #[error("The hash of a chunk was incorrect")]
    InvalidChecksum,
    #[error("This should never happen: {0}")]
    ShouldNotHaveFailed(&'static str),
    #[error("DataChunk content does not match the type it is being interpreted as")]
    TypeError,
}

pub type Result<T> = std::result::Result<T, PsDataChunkError>;
