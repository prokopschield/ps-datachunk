mod implementations;

use std::sync::Arc;

use ps_hash::Hash;
use ps_mbuf::Mbuf;

use crate::DataChunk;

#[derive(Clone, Copy)]
pub struct MbufDataChunk<'lt> {
    inner: &'lt Mbuf<'lt, Hash, u8>,
}

impl<'lt> From<&'lt Mbuf<'lt, Hash, u8>> for MbufDataChunk<'lt> {
    fn from(inner: &'lt Mbuf<'lt, Hash, u8>) -> Self {
        Self { inner }
    }
}

impl DataChunk for MbufDataChunk<'_> {
    fn data_ref(&self) -> &[u8] {
        self.inner
    }

    fn hash_ref(&self) -> &Hash {
        self.inner.get_metadata()
    }

    fn hash(&self) -> Hash {
        *self.inner.get_metadata()
    }

    /// Transforms this chunk into an [`OwnedDataChunk`]
    fn into_owned(self) -> crate::OwnedDataChunk {
        crate::OwnedDataChunk::from_data_and_hash_unchecked(Arc::from(self.data_ref()), self.hash())
    }
}
