mod implementations;

use crate::{DataChunk, PsDataChunkError, Result};
use bytes::Bytes;
use ps_hash::Hash;
use std::sync::Arc;

pub struct SharedDataChunk {
    data: Arc<[u8]>,
    hash: Hash,
}

impl SharedDataChunk {
    #[must_use]
    pub fn data(&self) -> Arc<[u8]> {
        self.data.clone()
    }

    #[must_use]
    pub const fn hash(&self) -> Hash {
        self.hash
    }
}

impl DataChunk for SharedDataChunk {
    fn data_ref(&self) -> &[u8] {
        &self.data
    }

    fn hash_ref(&self) -> &Hash {
        &self.hash
    }

    /// Transforms this [`DataChunk`] into [`Bytes`].
    fn into_bytes(self) -> Bytes {
        Bytes::from_owner(self.data)
    }

    /// Transforms this chunk into an [`OwnedDataChunk`]
    fn into_owned(self) -> crate::OwnedDataChunk {
        let Self { data, hash } = self;

        crate::OwnedDataChunk::from_data_and_hash_unchecked(data, hash)
    }
}

impl SharedDataChunk {
    #[must_use]
    pub const fn from_data_and_hash(data: Arc<[u8]>, hash: Hash) -> Self {
        Self { data, hash }
    }

    pub fn from_data(data: Arc<[u8]>) -> Result<Self> {
        let hash = ps_hash::hash(&data)?;

        Ok(Self::from_data_and_hash(data, hash))
    }
}

impl TryFrom<Arc<[u8]>> for SharedDataChunk {
    type Error = PsDataChunkError;

    fn try_from(data: Arc<[u8]>) -> Result<Self> {
        Self::from_data(data)
    }
}

impl TryFrom<&Arc<[u8]>> for SharedDataChunk {
    type Error = PsDataChunkError;

    fn try_from(data: &Arc<[u8]>) -> Result<Self> {
        Self::from_data(data.clone())
    }
}
