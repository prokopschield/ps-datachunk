use std::{ops::Deref, sync::Arc};

use bytes::Bytes;
use ps_buffer::{Buffer, SharedBuffer};
use ps_hash::{hash, Hash};

use crate::{utils::HASH_SIZE, DataChunk, EncryptedDataChunk, PsDataChunkError, Result};

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SerializedDataChunk {
    buffer: Buffer,
    hash: Arc<Hash>,
}

impl SerializedDataChunk {
    #[must_use]
    pub const fn data_length(&self) -> usize {
        self.buffer.len().saturating_sub(HASH_SIZE)
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buffer.len() <= HASH_SIZE
    }

    /// # Safety
    ///
    /// Called guarantees that `hash` is `hash(data)`
    ///
    /// This method does **NOT** verify `hash`!
    ///
    /// Call only if `hash` is surely known.
    pub fn from_parts<D>(data: D, hash: Arc<Hash>) -> Result<Self>
    where
        D: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let buffer_length = HASH_SIZE + data.len();

        let mut buffer = Buffer::with_capacity(buffer_length)?;

        buffer.extend_from_slice(hash.as_bytes())?;
        buffer.extend_from_slice(data)?;

        let chunk = Self { buffer, hash };

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
    pub fn from_data<D>(data: D) -> Result<Self>
    where
        D: AsRef<[u8]>,
    {
        let data = data.as_ref();

        Self::from_parts(data, hash(data)?.into())
    }

    /// Returns a reference to this [`SerializedDataChunk`]'s serialized bytes
    #[inline]
    #[must_use]
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

    #[inline]
    /// extracts the serialized `Buffer` from this `SerializedDataChunk`
    pub fn into_buffer(self) -> Buffer {
        self.buffer
    }

    #[inline]
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
        Ok(ps_cypher::encrypt(&self.buffer)?.into())
    }

    fn hash_ref(&self) -> &Hash {
        &self.hash
    }

    fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }

    /// Transforms this [`DataChunk`] into [`Bytes`].
    fn into_bytes(self) -> Bytes {
        Bytes::from_owner(SharedBuffer::from(self.buffer)).slice(HASH_SIZE..)
    }

    /// Transforms this chunk into an [`OwnedDataChunk`]
    fn into_owned(self) -> crate::OwnedDataChunk {
        let hash = self.hash();

        crate::OwnedDataChunk::from_data_and_hash(self, hash)
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
