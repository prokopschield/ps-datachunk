use std::{ops::Deref, sync::Arc};

use ps_buffer::Buffer;
use ps_hash::{hash, verify_hash_integrity, Hash};

use crate::{
    utils::{
        constants::{HASH_ALIGNMENT, HASH_SIZE, SIZE_ALIGNMENT, SIZE_SIZE},
        offsets::offsets,
        rounding::round_down,
    },
    DataChunk, DataChunkTrait, EncryptedDataChunk, OwnedDataChunk, PsDataChunkError, Result,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SerializedDataChunk {
    buffer: Buffer,
    hash: Arc<Hash>,
}

impl SerializedDataChunk {
    pub const fn data_offset(&self) -> usize {
        0
    }

    pub fn data_ref(&self) -> &[u8] {
        let offset_start = self.data_offset();
        let offset_end = offset_start + self.data_length();
        let byte_range = offset_start..offset_end;

        &self.buffer[byte_range]
    }

    pub const fn hash_offset(&self) -> usize {
        round_down(self.buffer.len() - HASH_SIZE, HASH_ALIGNMENT)
    }

    pub fn hash_ref(&self) -> &[u8] {
        let offset_start = self.hash_offset();
        let offset_end = offset_start + std::mem::size_of::<Hash>();
        let byte_range = offset_start..offset_end;

        &self.buffer[byte_range]
    }

    pub const fn length_offset(&self) -> usize {
        round_down(self.buffer.len() - SIZE_SIZE, SIZE_ALIGNMENT)
    }

    pub fn data_length(&self) -> usize {
        let offset_start = self.length_offset();
        let offset_end = offset_start + std::mem::size_of::<usize>();
        let byte_range = offset_start..offset_end;

        match self.buffer[byte_range].try_into() {
            Ok(bytes) => usize::from_le_bytes(bytes),
            _ => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.len() <= (HASH_SIZE + SIZE_SIZE)
    }

    /// # Safety
    ///
    /// Called guarantees that `hash` is `hash(data)`
    ///
    /// This method does **NOT** verify `hash`!
    ///
    /// Call only if `hash` is surely known.
    pub fn from_parts<D>(data: D, hash: Arc<Hash>) -> SerializedDataChunk
    where
        D: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let length = data.len();
        let length_bytes = length.to_le_bytes();

        let (hash_offset, size_offset, buffer_length) = offsets(length);

        let mut buffer = Buffer::alloc(buffer_length);

        buffer[0..length].copy_from_slice(data);
        buffer[hash_offset..hash_offset + hash.len()].copy_from_slice(hash.as_bytes());
        buffer[size_offset..size_offset + length_bytes.len()].copy_from_slice(&length_bytes);

        SerializedDataChunk { buffer, hash }
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

        if !verify_hash_integrity(hash) {
            return Err(PsDataChunkError::InvalidChecksum);
        }

        let hash = Hash::try_from(hash)?.into();

        Ok(Self::from_parts(data, hash))
    }

    /// Allocate a `SerializedDataChunk` containing `data`
    pub fn from_data<D>(data: D) -> SerializedDataChunk
    where
        D: AsRef<[u8]>,
    {
        let data = data.as_ref();

        Self::from_parts(data, hash(data).into())
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
        if buffer.len() < (HASH_SIZE + SIZE_SIZE) {
            // the `buffer` must include, at least, a `hash` and a `length`.
            return Err(PsDataChunkError::InvalidDataChunk)?;
        }

        let len_offset = round_down(buffer.len() - SIZE_SIZE, SIZE_ALIGNMENT);
        let hash_offset = round_down(len_offset - HASH_SIZE, HASH_ALIGNMENT);

        let length = usize::from_le_bytes(
            (&buffer[len_offset..len_offset + std::mem::size_of::<usize>()]).try_into()?,
        );

        if length > hash_offset {
            // `length` is obviously incorrect as `hash` would occupy the same bytes as `data`
            return Err(PsDataChunkError::InvalidLength(length))?;
        }

        let data = &buffer[0..length];
        let hash = ps_hash::hash(data).into();
        let chunk = Self { buffer, hash };

        if chunk.hash_ref() != chunk.hash.as_bytes() {
            // ensures data integrity
            return Err(PsDataChunkError::InvalidChecksum)?;
        }

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

impl DataChunkTrait for SerializedDataChunk {
    fn data_ref(&self) -> &[u8] {
        self.data_ref()
    }

    fn encrypt(&self) -> Result<EncryptedDataChunk> {
        OwnedDataChunk::encrypt_serialized_bytes(&self.buffer)
    }

    fn hash_ref(&self) -> &[u8] {
        self.hash_ref()
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

impl From<SerializedDataChunk> for DataChunk<'_> {
    fn from(chunk: SerializedDataChunk) -> Self {
        DataChunk::Serialized(chunk)
    }
}

impl From<DataChunk<'_>> for SerializedDataChunk {
    fn from(chunk: DataChunk) -> Self {
        if let DataChunk::Serialized(chunk) = chunk {
            chunk
        } else {
            chunk.serialize()
        }
    }
}
