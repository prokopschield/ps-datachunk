use ps_hash::Hash;

use crate::{BorrowedDataChunk, CowDataChunk, DataChunk, OwnedDataChunk};

impl DataChunk for CowDataChunk<'_> {
    fn borrow(&self) -> BorrowedDataChunk<'_> {
        BorrowedDataChunk::from_parts(self.data_ref(), self.hash())
    }

    fn data_ref(&self) -> &[u8] {
        match self {
            Self::Borrowed(chunk) => chunk.data_ref(),
            Self::Mbuf(chunk) => chunk.data_ref(),
            Self::Owned(chunk) => chunk.data_ref(),
        }
    }

    fn hash(&self) -> Hash {
        match self {
            Self::Borrowed(chunk) => chunk.hash(),
            Self::Mbuf(chunk) => chunk.hash(),
            Self::Owned(chunk) => chunk.hash(),
        }
    }

    fn hash_ref(&self) -> &ps_hash::Hash {
        match self {
            Self::Borrowed(chunk) => chunk.hash_ref(),
            Self::Mbuf(chunk) => chunk.hash_ref(),
            Self::Owned(chunk) => chunk.hash_ref(),
        }
    }

    fn into_bytes(self) -> bytes::Bytes {
        match self {
            Self::Borrowed(chunk) => chunk.into_bytes(),
            Self::Mbuf(chunk) => chunk.into_bytes(),
            Self::Owned(chunk) => chunk.into_bytes(),
        }
    }

    fn into_owned(self) -> OwnedDataChunk {
        match self {
            Self::Borrowed(chunk) => chunk.into_owned(),
            Self::Mbuf(chunk) => chunk.into_owned(),
            Self::Owned(chunk) => chunk.into_owned(),
        }
    }
}
