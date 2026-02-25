use crate::BorrowedDataChunk;

impl AsRef<[u8]> for BorrowedDataChunk<'_> {
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}
