mod implementations;

use crate::{BorrowedDataChunk, MbufDataChunk, OwnedDataChunk};

#[derive(Clone)]
pub enum CowDataChunk<'lt> {
    Borrowed(BorrowedDataChunk<'lt>),
    Mbuf(MbufDataChunk<'lt>),
    Owned(OwnedDataChunk),
}
