use crate::DataChunk;
use crate::EncryptedDataChunk;
use crate::Result;
use ps_hash::Hash;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// represents an owned chunk of data
pub struct OwnedDataChunk {
    hash: Arc<Hash>,
    data: Vec<u8>,
}

impl OwnedDataChunk {
    #[must_use]
    pub fn data_ref(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub fn hash_ref(&self) -> &Hash {
        &self.hash
    }

    #[must_use]
    pub fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }

    /// Creates an [`OwnedDataChunk`] from its constituent parts
    /// # Safety
    /// - `hash` must be the hash of `data`
    /// - use `from_data()` if you cannot ensure this
    #[inline]
    #[must_use]
    pub const fn from_parts(data: Vec<u8>, hash: Arc<Hash>) -> Self {
        Self { hash, data }
    }

    /// calculates the hash of `data` and returns an `OwnedDataChunk`
    pub fn from_data(data: Vec<u8>) -> Result<Self> {
        let hash = ps_hash::hash(&data)?;

        Ok(Self::from_parts(data, hash.into()))
    }

    #[inline]
    /// Encrypts a serialized [`DataChunk`].
    pub fn encrypt_serialized_bytes(bytes: &[u8]) -> Result<EncryptedDataChunk> {
        Ok(ps_cypher::encrypt(bytes)?.into())
    }

    #[inline]
    /// Encrypts this [`DataChunk`].
    pub fn encrypt(&self) -> Result<EncryptedDataChunk> {
        Self::encrypt_serialized_bytes(&self.serialize()?.into_buffer())
    }
}

impl DataChunk for OwnedDataChunk {
    fn data_ref(&self) -> &[u8] {
        self.data_ref()
    }
    fn hash_ref(&self) -> &Hash {
        self.hash_ref()
    }
    fn hash(&self) -> Arc<Hash> {
        self.hash()
    }
}
