use crate::{CowDataChunk, DataChunk};

impl AsRef<[u8]> for CowDataChunk<'_> {
    fn as_ref(&self) -> &[u8] {
        self.data_ref()
    }
}
