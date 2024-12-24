use super::{
    constants::{HASH_ALIGNMENT, HASH_SIZE, SIZE_ALIGNMENT, SIZE_SIZE},
    rounding::round_up,
};

pub const fn offsets(data_size: usize) -> (usize, usize, usize) {
    let hash_offset = round_up(data_size, HASH_ALIGNMENT);
    let size_offset = round_up(hash_offset + HASH_SIZE, SIZE_ALIGNMENT);
    let buffer_size = size_offset + SIZE_SIZE;

    (hash_offset, size_offset, buffer_size)
}
