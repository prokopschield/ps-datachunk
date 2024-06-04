use thiserror::Error;

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PsDataChunkError {
    #[error(transparent)]
    PsCypherError(#[from] ps_cypher::PsCypherError),
    #[error("Failed to convert a hash to [u8; 50]")]
    HashConversionError,
    #[error("Failed to serialize into an AlignedDataChunk")]
    SerializationError,
    #[error("Failed to deserialize from an AlignedDataChunk")]
    DeserializationError,
    #[error("The data chunk was not correctly layed out")]
    InvalidDataChunk,
    #[error("The hash of a chunk was incorrect")]
    InvalidChecksum,
    #[error("This should never happen: {0}")]
    ShouldNotHaveFailed(&'static str),
}

impl PsDataChunkError {
    pub fn map_option<T>(item: Option<T>, error: PsDataChunkError) -> Result<T, Self> {
        match item {
            Some(value) => Ok(value),
            None => Err(error),
        }
    }
    pub fn map_result<T, E>(item: Result<T, E>, error: PsDataChunkError) -> Result<T, Self> {
        match item {
            Ok(value) => Ok(value),
            Err(_) => Err(error),
        }
    }
}
