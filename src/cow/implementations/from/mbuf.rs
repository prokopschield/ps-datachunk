use crate::{CowDataChunk, MbufDataChunk};

impl<'lt> From<MbufDataChunk<'lt>> for CowDataChunk<'lt> {
    fn from(value: MbufDataChunk<'lt>) -> Self {
        Self::Mbuf(value)
    }
}
