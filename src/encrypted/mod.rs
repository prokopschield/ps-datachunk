use std::sync::Arc;

use crate::utils;
use crate::DataChunk;
use crate::Result;
use crate::SerializedDataChunk;
use ps_buffer::Buffer;
use ps_cypher::Encrypted;
use ps_hash::Hash;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// represents an encrypted chunk of data and the key needed to decrypt it
pub struct EncryptedDataChunk {
    data: Buffer,
    hash: Arc<Hash>,
    key: Arc<Hash>,
}

impl EncryptedDataChunk {
    /// Decrypts this `EncryptedDataChunk`.
    pub fn decrypt(&self) -> Result<SerializedDataChunk> {
        utils::decrypt(self.data_ref(), self.key.as_bytes())
    }
}

impl DataChunk for EncryptedDataChunk {
    fn data_ref(&self) -> &[u8] {
        &self.data
    }
    fn hash_ref(&self) -> &Hash {
        &self.hash
    }
    fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }
}

impl From<Encrypted> for EncryptedDataChunk {
    fn from(value: Encrypted) -> Self {
        Self {
            data: value.bytes,
            hash: value.hash,
            key: value.key,
        }
    }
}
