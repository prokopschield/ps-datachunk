use std::sync::Arc;

use bytes::Bytes;

use crate::{OwnedDataChunk, PsDataChunkError, Result};

impl From<OwnedDataChunk> for Bytes {
    fn from(value: OwnedDataChunk) -> Self {
        value.data
    }
}

impl From<&OwnedDataChunk> for Bytes {
    fn from(value: &OwnedDataChunk) -> Self {
        value.bytes()
    }
}

impl TryFrom<Arc<[u8]>> for OwnedDataChunk {
    type Error = PsDataChunkError;

    fn try_from(value: Arc<[u8]>) -> Result<Self> {
        let hash = ps_hash::hash(&value)?;

        Ok(Self::from_data_and_hash_unchecked(value, hash))
    }
}

impl TryFrom<&Arc<[u8]>> for OwnedDataChunk {
    type Error = PsDataChunkError;

    fn try_from(value: &Arc<[u8]>) -> Result<Self> {
        let hash = ps_hash::hash(value)?;

        Ok(Self::from_data_and_hash_unchecked(value.clone(), hash))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use bytes::Bytes;

    use crate::{OwnedDataChunk, Result};

    #[test]
    fn test_owned_datachunk_to_bytes_from_ref() -> Result<()> {
        let original_data = vec![1, 2, 3, 4, 5];
        let data_chunk = OwnedDataChunk::from_data(original_data.clone())?;

        let bytes = Bytes::from(&data_chunk);

        assert_eq!(bytes.as_ref(), original_data);

        Ok(())
    }

    #[test]
    fn test_owned_datachunk_try_from_arc() -> Result<()> {
        let original_data: Arc<[u8]> = Arc::from(vec![9, 8, 7, 6]);
        let data_chunk = OwnedDataChunk::try_from(original_data.clone())?;

        assert_eq!(data_chunk.data_ref(), original_data.as_ref());
        assert_eq!(data_chunk.hash(), ps_hash::hash(&original_data)?);

        Ok(())
    }

    #[test]
    fn test_owned_datachunk_try_from_arc_ref() -> Result<()> {
        let original_data: Arc<[u8]> = Arc::from(vec![9, 8, 7, 6]);
        let data_chunk = OwnedDataChunk::try_from(&original_data)?;

        assert_eq!(data_chunk.data_ref(), original_data.as_ref());
        assert_eq!(data_chunk.hash(), ps_hash::hash(&original_data)?);

        Ok(())
    }
}
