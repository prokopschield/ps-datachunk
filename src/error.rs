use ps_cypher::PsCypherError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PsDataChunkError {
    PsCypherError(ps_cypher::PsCypherError),
    HashConversionError,
    SerializationError,
    DeserializationError,
    InvalidChecksum,
    Other,
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

impl From<PsCypherError> for PsDataChunkError {
    fn from(error: PsCypherError) -> Self {
        Self::PsCypherError(error)
    }
}
