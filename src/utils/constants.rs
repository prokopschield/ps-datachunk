use ps_hash::Hash;

use super::rounding::round_up;

/// 8 bytes
pub const SIZE_ALIGNMENT: usize = 3;

/// 64 bytes
pub const HASH_SIZE: usize = round_up(std::mem::size_of::<Hash>(), SIZE_ALIGNMENT);
