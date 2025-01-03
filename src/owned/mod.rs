pub mod deserializer;
pub mod serializer;

use crate::utils::offsets;
use crate::DataChunk;
use crate::DataChunkTrait;
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
    pub fn data_ref(&self) -> &[u8] {
        &self.data
    }

    pub fn hash_ref(&self) -> &[u8] {
        self.hash.as_bytes()
    }

    pub fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }

    /// Creates an OwnedDataChunk from its constituent parts
    /// # Safety
    /// - `hash` must be the hash of `data`
    /// - use `from_data()` if you cannot ensure this
    #[inline(always)]
    pub fn from_parts(data: Vec<u8>, hash: Arc<Hash>) -> Self {
        Self { data, hash }
    }

    /// calculates the hash of `data` and returns an `OwnedDataChunk`
    pub fn from_data(data: Vec<u8>) -> Self {
        let hash = ps_hash::hash(&data);

        Self::from_parts(data, hash.into())
    }

    /// creates an `OwnedDataChunk` with given `data` and `hash`
    pub fn from_data_ref_and_hash(data: &[u8], hash: Arc<Hash>) -> Self {
        let reserved_size = offsets(data.len()).2;
        let mut data_vec = Vec::with_capacity(reserved_size);

        data_vec.extend_from_slice(data);

        Self::from_parts(data_vec, hash)
    }

    /// creates an `OwnedDataChunk` with given `data`
    pub fn from_data_ref(data: &[u8]) -> Self {
        Self::from_data_ref_and_hash(data, ps_hash::hash(data).into())
    }

    #[inline(always)]
    /// - converts a `Vec<u8>` into an `OwnedDataChunk`
    /// - performs hash validation
    pub fn deserialize_from(data: Vec<u8>) -> Result<Self> {
        let (data, hash) = deserializer::deserialize_vec_to_parts(data)?;

        Ok(Self {
            data,
            hash: hash.into(),
        })
    }

    #[inline(always)]
    /// Copies `data` into a new `Vec<u8>` and deserializes it into an `OwnedDataChunk`.
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        Self::deserialize_from(data.to_vec())
    }

    #[inline(always)]
    /// Encrypts a serialized [DataChunk].
    pub fn encrypt_serialized_bytes(bytes: &[u8]) -> Result<EncryptedDataChunk> {
        Ok(ps_cypher::encrypt(bytes)?.into())
    }

    #[inline(always)]
    /// Encrypts this [DataChunk].
    pub fn encrypt(&self) -> Result<EncryptedDataChunk> {
        Self::encrypt_serialized_bytes(&self.serialize().into_buffer())
    }

    #[inline(always)]
    /// Encrypts this [DataChunk].
    /// - optimized by using `self.data` as the serialization buffer
    pub fn encrypt_mut(&mut self) -> Result<EncryptedDataChunk> {
        let data_length = self.data.len();

        serializer::serialize_vec_with_known_hash(&mut self.data, self.hash.as_bytes());

        let encrypted = Self::encrypt_serialized_bytes(&self.data);

        self.data.truncate(data_length);

        encrypted
    }
}

impl DataChunkTrait for OwnedDataChunk {
    fn data_ref(&self) -> &[u8] {
        self.data_ref()
    }
    fn hash_ref(&self) -> &[u8] {
        self.hash_ref()
    }
    fn hash(&self) -> Arc<Hash> {
        self.hash()
    }
}

impl<'lt> From<DataChunk<'lt>> for OwnedDataChunk {
    fn from(value: DataChunk<'lt>) -> Self {
        match value {
            DataChunk::Owned(owned) => owned,
            _ => OwnedDataChunk::from_data_ref_and_hash(value.data_ref(), value.hash()),
        }
    }
}
