use rkyv::Archive;

use crate::{DataChunk, TypedDataChunk};

impl<D, T> AsRef<[u8]> for TypedDataChunk<D, T>
where
    D: DataChunk,
    T: Archive,
{
    fn as_ref(&self) -> &[u8] {
        self.chunk.data_ref()
    }
}
