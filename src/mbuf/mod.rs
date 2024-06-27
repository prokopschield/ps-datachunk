use crate::{DataChunkTrait, HashCow};
use ps_hash::Hash;
use ps_mbuf::Mbuf;

pub struct MbufDataChunk<'lt> {
    inner: &'lt Mbuf<'lt, Hash, u8>,
}

impl<'lt> From<&'lt Mbuf<'lt, Hash, u8>> for MbufDataChunk<'lt> {
    fn from(inner: &'lt Mbuf<'lt, Hash, u8>) -> Self {
        Self { inner }
    }
}

impl<'lt> DataChunkTrait for MbufDataChunk<'lt> {
    fn data_ref(&self) -> &[u8] {
        &self.inner
    }

    fn hash_ref(&self) -> &[u8] {
        self.inner.get_metadata().as_bytes()
    }

    fn hash(&self) -> HashCow<'lt> {
        HashCow::Borrowed(self.inner.get_metadata())
    }
}
