use std::sync::Arc;

use crate::utils;
use crate::DataChunkTrait;
use crate::OwnedDataChunk;
use crate::Result;
use crate::SerializedDataChunk;
use ps_cypher::Encrypted;
use ps_hash::Hash;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// represents an encrypted chunk of data and the key needed to decrypt it
pub struct EncryptedDataChunk {
    pub chunk: OwnedDataChunk,
    pub key: std::sync::Arc<Hash>,
}

impl EncryptedDataChunk {
    /// Decrypts this `EncryptedDataChunk`.
    pub fn decrypt(&self) -> Result<SerializedDataChunk> {
        utils::decrypt(self.chunk.data_ref(), self.key.as_bytes())
    }
}

impl DataChunkTrait for EncryptedDataChunk {
    fn data_ref(&self) -> &[u8] {
        self.chunk.data_ref()
    }
    fn hash_ref(&self) -> &[u8] {
        self.chunk.hash_ref()
    }
    fn hash(&self) -> Arc<Hash> {
        self.chunk.hash()
    }
}

impl From<Encrypted> for EncryptedDataChunk {
    fn from(value: Encrypted) -> Self {
        Self {
            chunk: OwnedDataChunk::from_parts(value.bytes, value.hash),
            key: value.key,
        }
    }
}
