use crate::{CowDataChunk, OwnedDataChunk};

impl From<OwnedDataChunk> for CowDataChunk<'_> {
    fn from(value: OwnedDataChunk) -> Self {
        Self::Owned(value)
    }
}
