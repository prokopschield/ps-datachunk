use ps_hash::Hash;

use super::rounding::round_up;

pub const HASH_SIZE: usize = round_up(std::mem::size_of::<Hash>(), 3);
pub const SIZE_SIZE: usize = round_up(std::mem::size_of::<usize>(), 3);
