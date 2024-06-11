use crate::{AlignedDataChunk, DataChunk, OwnedDataChunk};

pub trait AbstractDataChunk {
    fn data(&self) -> &[u8];
    fn hash(&self) -> &[u8];
}

impl<'lt> AbstractDataChunk for DataChunk<'lt> {
    fn data(&self) -> &[u8] {
        match self {
            Self::Aligned(aligned) => aligned.data(),
            Self::Mbuf(mbuf) => mbuf,
            DataChunk::Owned(owned) => &owned.data,
        }
    }
    fn hash(&self) -> &[u8] {
        match self {
            Self::Aligned(aligned) => aligned.hash_ref(),
            Self::Mbuf(mbuf) => mbuf.get_metadata(),
            DataChunk::Owned(owned) => &owned.hash,
        }
    }
}

impl AbstractDataChunk for AlignedDataChunk {
    fn data(&self) -> &[u8] {
        self.data()
    }
    fn hash(&self) -> &[u8] {
        self.hash_ref()
    }
}

impl AbstractDataChunk for OwnedDataChunk {
    fn data(&self) -> &[u8] {
        &self.data
    }
    fn hash(&self) -> &[u8] {
        &self.hash
    }
}
