use ps_buffer::Buffer;
use ps_hash::Hash;

use crate::utils::{
    constants::{HASH_ALIGNMENT, HASH_SIZE, SIZE_ALIGNMENT, SIZE_SIZE},
    rounding::round_down,
};

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
}
