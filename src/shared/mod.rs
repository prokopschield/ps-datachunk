use crate::DataChunk;
use crate::DataChunkTrait;
use ps_hash::Hash;
use std::sync::Arc;

pub struct SharedDataChunk {
    data: Arc<[u8]>,
    hash: Arc<Hash>,
}

impl SharedDataChunk {
    pub fn data(&self) -> Arc<[u8]> {
        self.data.clone()
    }

    pub fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }
}

impl DataChunkTrait for SharedDataChunk {
    fn data_ref(&self) -> &[u8] {
        &self.data
    }

    fn hash_ref(&self) -> &[u8] {
        self.hash.as_bytes()
    }

    fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }

    fn to_owned(&self) -> crate::OwnedDataChunk {
        crate::OwnedDataChunk::from_data_ref_and_hash(self.data_ref(), self.hash())
    }
}

impl SharedDataChunk {
    pub fn from_data_and_hash(data: Arc<[u8]>, hash: Arc<Hash>) -> Self {
        Self { data, hash }
    }

    pub fn from_data(data: Arc<[u8]>) -> Self {
        let hash = Arc::from(ps_hash::hash(&data));

        Self::from_data_and_hash(data, hash)
    }
}

impl From<Arc<[u8]>> for SharedDataChunk {
    fn from(data: Arc<[u8]>) -> Self {
        Self::from_data(data)
    }
}

impl From<&Arc<[u8]>> for SharedDataChunk {
    fn from(data: &Arc<[u8]>) -> Self {
        Self::from_data(data.clone())
    }
}

impl<'lt> From<Arc<[u8]>> for DataChunk<'lt> {
    fn from(data: Arc<[u8]>) -> Self {
        DataChunk::Shared(data.into())
    }
}

impl<'lt> From<&Arc<[u8]>> for DataChunk<'lt> {
    fn from(data: &Arc<[u8]>) -> Self {
        DataChunk::Shared(data.into())
    }
}
