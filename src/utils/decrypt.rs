use crate::*;

pub fn decrypt<D, K>(encrypted: D, key: K, compressor: &Compressor) -> Result<SerializedDataChunk>
where
    D: AsRef<[u8]>,
    K: AsRef<[u8]>,
{
    let buffer = ps_cypher::decrypt(encrypted.as_ref(), key.as_ref(), compressor)?;

    let chunk = SerializedDataChunk::from_serialized_buffer(buffer)?;

    Ok(chunk)
}
