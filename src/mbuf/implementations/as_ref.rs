use crate::MbufDataChunk;

impl AsRef<[u8]> for MbufDataChunk<'_> {
    fn as_ref(&self) -> &[u8] {
        self.inner
    }
}
