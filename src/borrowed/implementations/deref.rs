use std::ops::Deref;

use crate::BorrowedDataChunk;

impl Deref for BorrowedDataChunk<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.data
    }
}
