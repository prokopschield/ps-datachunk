use ps_buffer::Buffer;
use ps_hash::{hash, Hash};

use crate::{
    utils::{
        constants::{HASH_ALIGNMENT, HASH_SIZE, SIZE_ALIGNMENT, SIZE_SIZE},
        offsets::offsets,
        rounding::round_down,
    },
    DataChunkTrait,
};

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SerializedDataChunk {
    buffer: Buffer,
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

    pub fn from_parts<D, H>(data: D, hash: H) -> SerializedDataChunk
    where
        D: AsRef<[u8]>,
        H: AsRef<[u8]>,
    {
        let data = data.as_ref();
        let hash = hash.as_ref();
        let length = data.len();
        let length_bytes = length.to_le_bytes();

        let (hash_offset, size_offset, buffer_length) = offsets(length);

        let mut buffer = Buffer::alloc(buffer_length);

        buffer[0..length].copy_from_slice(data);
        buffer[hash_offset..hash_offset + hash.len()].copy_from_slice(hash);
        buffer[size_offset..size_offset + length_bytes.len()].copy_from_slice(&length_bytes);

        SerializedDataChunk { buffer }
    }

    pub fn from_data(data: &[u8]) -> SerializedDataChunk {
        Self::from_parts(data, &hash(data))
    }
}

impl DataChunkTrait for SerializedDataChunk {
    fn data_ref(&self) -> &[u8] {
        self.data_ref()
    }

    fn hash_ref(&self) -> &[u8] {
        self.hash_ref()
    }
}
