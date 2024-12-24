use crate::utils::offsets::offsets;

#[inline(always)]
pub fn serialize_vec_with_parameters(
    data: &mut Vec<u8>,
    hash: &[u8],
    data_length: usize,
    hash_offset: usize,
    size_offset: usize,
) -> () {
    data.extend_from_slice(&vec![0u8; hash_offset - data.len()]);
    data.extend_from_slice(hash);
    data.extend_from_slice(&vec![0u8; size_offset - data.len()]);
    data.extend_from_slice(&data_length.to_le_bytes());
}

#[inline(always)]
pub fn serialize_vec_with_known_hash(data: &mut Vec<u8>, hash: &[u8]) -> () {
    let data_length = data.len();
    let (hash_offset, size_offset, _) = offsets(data_length);

    serialize_vec_with_parameters(data, hash, data_length, hash_offset, size_offset)
}

#[inline]
pub fn serialize_vec(data: &mut Vec<u8>) -> () {
    serialize_vec_with_known_hash(data, ps_hash::hash(&data).as_bytes())
}

#[inline]
pub fn serialize_bytes_with_known_hash(data: &[u8], hash: &[u8]) -> Vec<u8> {
    let data_length = data.len();
    let (hash_offset, size_offset, size) = offsets(data_length);
    let mut serialized = Vec::with_capacity(size);

    serialized.extend_from_slice(data);

    serialize_vec_with_parameters(&mut serialized, hash, data_length, hash_offset, size_offset);

    return serialized;
}

#[inline]
pub fn serialize_bytes(data: &[u8]) -> Vec<u8> {
    serialize_bytes_with_known_hash(data, ps_hash::hash(data).as_bytes())
}
