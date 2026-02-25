use crate::EncryptedDataChunk;

impl AsRef<[u8]> for EncryptedDataChunk {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}
