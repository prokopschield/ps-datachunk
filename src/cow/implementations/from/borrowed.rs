use crate::{BorrowedDataChunk, CowDataChunk};

impl<'lt> From<BorrowedDataChunk<'lt>> for CowDataChunk<'lt> {
    fn from(value: BorrowedDataChunk<'lt>) -> Self {
        Self::Borrowed(value)
    }
}
