use std::ops::Deref;

use crate::MbufDataChunk;

impl Deref for MbufDataChunk<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
