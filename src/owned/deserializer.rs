use ps_hash::Hash;

use crate::{
    aligned::{offsets, rdown, HSIZE, SSIZE},
    PsDataChunkError, Result,
};

pub fn deserialize_vec_to_parts(mut data: Vec<u8>) -> Result<(Vec<u8>, Hash)> {
    if data.len() < HSIZE + SSIZE {
        return Err(PsDataChunkError::InvalidDataChunk);
    }

    let data_size_offset = rdown(data.len() - 1, 3);

    if data_size_offset + std::mem::size_of::<usize>() < data.len() {
        return Err(PsDataChunkError::InvalidDataChunk);
    }

    let data_size_bytes = &data[data_size_offset..data_size_offset + std::mem::size_of::<usize>()];

    let data_size = usize::from_le_bytes(data_size_bytes.try_into().map_err(|_| {
        PsDataChunkError::ShouldNotHaveFailed(
            "Length is hard-coded so conversion should never fail.",
        )
    })?);

    let (hash_offset, check_length_offset, _) = offsets(data_size, HSIZE);

    if check_length_offset != data_size_offset {
        return Err(PsDataChunkError::InvalidDataChunk);
    }

    let computed_hash = ps_hash::hash(&data[0..data_size]);
    let check_hash_bytes = &data[hash_offset..hash_offset + std::mem::size_of::<Hash>()];

    if computed_hash.as_bytes() != check_hash_bytes {
        return Err(PsDataChunkError::InvalidChecksum);
    }

    data.truncate(data_size);

    Ok((data, computed_hash))
}

pub fn deserialize_bytes_to_parts(data: &[u8]) -> Result<(Vec<u8>, Hash)> {
    deserialize_vec_to_parts(data.to_vec())
}
