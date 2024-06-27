use crate::OwnedDataChunk;
use crate::PsDataChunkError;
use ps_cypher::Compressor;
use ps_hash::Hash;

/// represents an encrypted chunk of data and the key needed to decrypt it
pub struct EncryptedDataChunk {
    pub chunk: OwnedDataChunk,
    pub key: std::sync::Arc<Hash>,
}

impl EncryptedDataChunk {
    /// Decrypts this `EncryptedDataChunk`.
    pub fn decrypt(&self, compressor: &Compressor) -> Result<OwnedDataChunk, PsDataChunkError> {
        OwnedDataChunk::decrypt(&self.chunk, self.key.as_bytes(), compressor)
    }
}
