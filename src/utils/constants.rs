use ps_hash::Hash;

use super::rounding::round_up;

/// 16 bytes
pub const HASH_ALIGNMENT: usize = 4;

/// 8 bytes
pub const SIZE_ALIGNMENT: usize = 3;

/// 54 bytes
pub const HASH_SIZE: usize = round_up(std::mem::size_of::<Hash>(), SIZE_ALIGNMENT);

/// 8 bytes on x86-64
pub const SIZE_SIZE: usize = round_up(std::mem::size_of::<usize>(), SIZE_ALIGNMENT);
