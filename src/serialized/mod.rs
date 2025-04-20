use std::{ops::Deref, sync::Arc};

use ps_buffer::Buffer;
use ps_hash::{hash, Hash};

use crate::{
    utils::HASH_SIZE, DataChunk, EncryptedDataChunk, OwnedDataChunk, PsDataChunkError, Result,
};

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SerializedDataChunk {
    buffer: Buffer,
    hash: Arc<Hash>,
}

impl SerializedDataChunk {
    pub fn data_length(&self) -> usize {
        self.buffer.len().saturating_sub(HASH_SIZE)
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.len() <= HASH_SIZE
    }

    /// # Safety
    ///
    /// Called guarantees that `hash` is `hash(data)`
    ///
    /// This method does **NOT** verify `hash`!
    ///
    /// Call only if `hash` is surely known.
    pub fn from_parts<D>(data: D, hash: Arc<Hash>) -> Result<SerializedDataChunk>
    where
        D: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let buffer_length = HASH_SIZE + data.len();

        let mut buffer = Buffer::with_capacity(buffer_length)?;

        buffer.extend_from_slice(hash.as_bytes())?;
        buffer.extend_from_slice(data)?;

        let chunk = SerializedDataChunk { buffer, hash };

        Ok(chunk)
    }

    /// # Safety
    ///
    /// Called guarantees that `hash` is `hash(data)`
    ///
    /// This method does **NOT** verify `hash`!
    ///
    /// This method only verifies the internal checksum of `hash`, and will return `Err(PsDataChunkError::InvalidChecksum)` if this is invalid.
    pub fn try_from_parts<D, H>(data: D, hash: H) -> Result<Self>
    where
        D: AsRef<[u8]>,
        H: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let hash = hash.as_ref();

        let hash = Hash::try_from(hash)?.into();

        Self::from_parts(data, hash)
    }

    /// Allocate a `SerializedDataChunk` containing `data`
    pub fn from_data<D>(data: D) -> Result<SerializedDataChunk>
    where
        D: AsRef<[u8]>,
    {
        let data = data.as_ref();

        Self::from_parts(data, hash(data)?.into())
    }

    /// Returns a reference to this SerializedDataChunk's serialized bytes
    #[inline(always)]
    pub fn serialized_bytes(&self) -> &[u8] {
        &self.buffer
    }

    /// Constructs a `SerializedDataChunk` from a serialized buffer.
    ///
    /// `buffer` is validated to be interpretable as a `SerializedDataChunk`,
    /// and its `hash` is recalculated and verified. However, other things,
    /// such as padding and buffer length, are not validated.
    pub fn from_serialized_buffer(buffer: Buffer) -> Result<Self> {
        if buffer.len() < HASH_SIZE {
            return Err(PsDataChunkError::InvalidDataChunk);
        }

        let hash = &buffer[..HASH_SIZE];
        let data = &buffer[HASH_SIZE..];
        let calculated_hash = ps_hash::hash(data)?;

        if hash != calculated_hash.as_bytes() {
            return Err(PsDataChunkError::InvalidHash);
        }

        let chunk = Self {
            buffer,
            hash: Arc::from(calculated_hash),
        };

        Ok(chunk)
    }

    #[inline(always)]
    /// extracts the serialized `Buffer` from this `SerializedDataChunk`
    pub fn into_buffer(self) -> Buffer {
        self.buffer
    }

    #[inline(always)]
    /// extracts the serialized `Buffer` and `Hash` from this `SerializedDataChunk`
    pub fn into_parts(self) -> (Buffer, Arc<Hash>) {
        (self.buffer, self.hash)
    }
}

impl DataChunk for SerializedDataChunk {
    fn data_ref(&self) -> &[u8] {
        &self.buffer[HASH_SIZE..]
    }

    fn encrypt(&self) -> Result<EncryptedDataChunk> {
        OwnedDataChunk::encrypt_serialized_bytes(&self.buffer)
    }

    fn hash_ref(&self) -> &Hash {
        &self.hash
    }

    fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }
}

impl AsRef<[u8]> for SerializedDataChunk {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl Deref for SerializedDataChunk {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data_ref()
    }
}
