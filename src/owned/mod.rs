pub mod deserializer;
pub mod serializer;

use crate::Compressor;
use crate::EncryptedDataChunk;
use crate::PsDataChunkError;

#[derive(rkyv::Archive, rkyv::Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// represents an owned chunk of data
pub struct OwnedDataChunk {
    pub hash: [u8; 50],
    pub data: Vec<u8>,
}

impl OwnedDataChunk {
    #[inline(always)]
    /// converts this `OwnedDataChunk` into a `Vec<u8>`
    /// - extends `self.hash`
    /// - returns `self.data`
    pub fn serialize_into(mut self) -> Vec<u8> {
        serializer::serialize_vec_with_known_hash(&mut self.data, &self.hash);

        return self.data;
    }

    #[inline(always)]
    /// serializes this `OwnedDataChunk` into a newly allocated `Vec<u8>`
    /// - allocated a new `Vec<u8>`
    /// - copies `self.data` into the new `Vec<u8>`
    /// - copies `self.hash` into the new `Vec<u8>`
    /// - returns the new `Vec<u8>`
    pub fn serialize(&self) -> Vec<u8> {
        serializer::serialize_bytes_with_known_hash(&self.data, &self.hash)
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

        serializer::serialize_vec_with_known_hash(&mut self.data, &self.hash);

        let encrypted = Self::encrypt_bytes(&self.data, compressor);

        self.data.truncate(data_length);

        return encrypted;
    }
}
