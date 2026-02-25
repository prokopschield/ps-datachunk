use crate::OwnedDataChunk;

impl AsRef<[u8]> for OwnedDataChunk {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}
