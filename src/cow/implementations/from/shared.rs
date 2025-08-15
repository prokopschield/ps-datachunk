use crate::{CowDataChunk, DataChunk, SharedDataChunk};

impl From<SharedDataChunk> for CowDataChunk<'_> {
    fn from(value: SharedDataChunk) -> Self {
        Self::Owned(value.into_owned())
    }
}
