pub mod deserializer;
pub mod serializer;

use crate::aligned::rup;
use crate::aligned::HSIZE;
use crate::Compressor;
use crate::DataChunkTrait;
use crate::EncryptedDataChunk;
use crate::PsDataChunkError;
use ps_hash::Hash;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// represents an owned chunk of data
pub struct OwnedDataChunk {
    hash: Arc<Hash>,
    data: Vec<u8>,
}

impl OwnedDataChunk {
    pub fn data_ref(&self) -> &[u8] {
        &self.data
    }

    pub fn hash_ref(&self) -> &[u8] {
        self.hash.as_bytes()
    }

    /// Creates an OwnedDataChunk from its constituent parts
    /// # Safety
    /// - `hash` must be the hash of `data`
    /// - use `from_data()` if you cannot ensure this
    pub fn from_parts(data: Vec<u8>, hash: Hash) -> Self {
        Self {
            data,
            hash: hash.into(),
        }
    }

    /// calculates the hash of `data` and returns an `OwnedDataChunk`
    pub fn from_data(data: Vec<u8>) -> Self {
        let hash = ps_hash::hash(&data);

        Self::from_parts(data, hash)
    }

    /// creates an `OwnedDataChunk` with given `data`
    pub fn from_data_ref(data: &[u8]) -> Self {
        let reserved_size = rup(data.len(), 6) + rup(HSIZE, 6);
        let mut data_vec = Vec::with_capacity(reserved_size);

        data_vec.extend_from_slice(data);

        Self::from_data(data_vec)
    }

    #[inline(always)]
    /// converts this `OwnedDataChunk` into a `Vec<u8>`
    /// - extends `self.hash`
    /// - returns `self.data`
    pub fn serialize_into(mut self) -> Vec<u8> {
        serializer::serialize_vec_with_known_hash(&mut self.data, self.hash.as_bytes());

        return self.data;
    }

    #[inline(always)]
    /// serializes this `OwnedDataChunk` into a newly allocated `Vec<u8>`
    /// - allocated a new `Vec<u8>`
    /// - copies `self.data` into the new `Vec<u8>`
    /// - copies `self.hash` into the new `Vec<u8>`
    /// - returns the new `Vec<u8>`
    pub fn serialize(&self) -> Vec<u8> {
        serializer::serialize_bytes_with_known_hash(&self.data, self.hash_ref())
    }

    #[inline(always)]
    /// - converts a `Vec<u8>` into an `OwnedDataChunk`
    /// - performs hash validation
    pub fn deserialize_from(data: Vec<u8>) -> Result<Self, PsDataChunkError> {
        let (data, hash) = deserializer::deserialize_vec_to_parts(data)?;

        Ok(Self {
            data,
            hash: hash.into(),
        })
    }

    #[inline(always)]
    /// Copies `data` into a new `Vec<u8>` and deserializes it into an `OwnedDataChunk`.
    pub fn deserialize(data: &[u8]) -> Result<Self, PsDataChunkError> {
        Self::deserialize_from(data.to_vec())
    }

    #[inline(always)]
    /// Decrypts into an `OwnedDataChunk` with the given `key`
    /// - performs hash validation
    /// - fails if `key` not correct
    pub fn decrypt_bytes(
        encrypted: &[u8],
        key: &[u8],
        compressor: &Compressor,
    ) -> Result<Self, PsDataChunkError> {
        let decrypted = ps_cypher::decrypt(encrypted, key, compressor)?;

        Self::deserialize_from(decrypted)
    }

    #[inline(always)]
    /// Decrypts an `OwnedDataChunk` with the given `key`.
    /// - performs hash validation
    /// - fails if `key` not correct
    pub fn decrypt(&self, key: &[u8], compressor: &Compressor) -> Result<Self, PsDataChunkError> {
        Self::decrypt_bytes(&self.data, key, compressor)
    }

    #[inline(always)]
    /// Encrypts a serialized [DataChunk].
    pub fn encrypt_bytes(
        bytes: &[u8],
        compressor: &Compressor,
    ) -> Result<EncryptedDataChunk, PsDataChunkError> {
        let encrypted = ps_cypher::encrypt(bytes, compressor)?;

        Ok(EncryptedDataChunk {
            chunk: OwnedDataChunk {
                data: encrypted.bytes,
                hash: encrypted.hash.into(),
            },
            key: encrypted.key.into(),
        })
    }

    #[inline(always)]
    /// Encrypts this [DataChunk].
    pub fn encrypt(&self, compressor: &Compressor) -> Result<EncryptedDataChunk, PsDataChunkError> {
        Self::encrypt_bytes(&self.serialize(), compressor)
    }

    #[inline(always)]
    /// Encrypts this [DataChunk].
    /// - optimized by using `self.data` as the serialization buffer
    pub fn encrypt_mut(
        &mut self,
        compressor: &Compressor,
    ) -> Result<EncryptedDataChunk, PsDataChunkError> {
        let data_length = self.data.len();

        serializer::serialize_vec_with_known_hash(&mut self.data, self.hash.as_bytes());

        let encrypted = Self::encrypt_bytes(&self.data, compressor);

        self.data.truncate(data_length);

        return encrypted;
    }
}

impl DataChunkTrait for OwnedDataChunk {
    fn data_ref(&self) -> &[u8] {
        self.data_ref()
    }
    fn hash_ref(&self) -> &[u8] {
        self.hash_ref()
    }
}
