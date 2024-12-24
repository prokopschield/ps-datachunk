use ps_buffer::Buffer;

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

    pub const fn hash_offset(&self) -> usize {
        round_down(self.buffer.len() - HASH_SIZE, HASH_ALIGNMENT)
    }

    pub const fn length_offset(&self) -> usize {
        round_down(self.buffer.len() - SIZE_SIZE, SIZE_ALIGNMENT)
    }
}
