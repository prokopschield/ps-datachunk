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

    fn hash(&self) -> crate::HashCow {
        crate::HashCow::from_arc(self.hash.clone())
    }
}
