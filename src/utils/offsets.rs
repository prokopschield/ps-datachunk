use super::{
    constants::{HASH_SIZE, SIZE_SIZE},
    rounding::round_up,
};

pub const fn offsets(data_size: usize) -> (usize, usize, usize) {
    let hash_offset = round_up(data_size, 4);
    let size_offset = round_up(hash_offset + HASH_SIZE, 3);
    let buffer_size = round_up(size_offset + SIZE_SIZE, 4);

    (hash_offset, size_offset, buffer_size)
}
