use crate::SharedDataChunk;

impl AsRef<[u8]> for SharedDataChunk {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}
