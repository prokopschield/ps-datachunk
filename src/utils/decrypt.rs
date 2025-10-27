use ps_hash::Hash;

use crate::{Result, SerializedDataChunk};

pub fn decrypt(encrypted: impl AsRef<[u8]>, key: &Hash) -> Result<SerializedDataChunk> {
    let buffer = ps_cypher::decrypt(encrypted.as_ref(), key)?;

    let chunk = SerializedDataChunk::from_serialized_buffer(buffer)?;

    Ok(chunk)
}
