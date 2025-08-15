use std::ops::Deref;

use crate::CowDataChunk;

impl Deref for CowDataChunk<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(chunk) => chunk,
            Self::Mbuf(chunk) => chunk,
            Self::Owned(chunk) => chunk,
        }
    }
}
