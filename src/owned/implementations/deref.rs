use std::ops::Deref;

use crate::OwnedDataChunk;

impl Deref for OwnedDataChunk {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
