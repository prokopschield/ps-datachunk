use bytes::Bytes;

use crate::OwnedDataChunk;

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

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::OwnedDataChunk;
    use crate::Result;

    #[test]
    fn test_owned_datachunk_to_bytes_from_ref() -> Result<()> {
        let original_data = vec![1, 2, 3, 4, 5];
        let data_chunk = OwnedDataChunk::from_data(original_data.clone())?;

        let bytes = Bytes::from(&data_chunk);

        assert_eq!(bytes.as_ref(), original_data);

        Ok(())
    }
}
