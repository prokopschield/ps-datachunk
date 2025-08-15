mod implementations;

use std::sync::Arc;

use ps_hash::Hash;

use crate::{DataChunk, Result};

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BorrowedDataChunk<'lt> {
    data: &'lt [u8],
    hash: Arc<Hash>,
}

impl<'lt> BorrowedDataChunk<'lt> {
    #[must_use]
    pub const fn from_parts(data: &'lt [u8], hash: Arc<Hash>) -> Self {
        Self { data, hash }
    }

    pub fn from_data(data: &'lt [u8]) -> Result<Self> {
        let hash = ps_hash::hash(data)?;

        Ok(Self::from_parts(data, hash.into()))
    }
}

impl DataChunk for BorrowedDataChunk<'_> {
    fn data_ref(&self) -> &[u8] {
        self.data
    }
    fn hash_ref(&self) -> &Hash {
        &self.hash
    }
    fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }

    fn borrow(&self) -> BorrowedDataChunk<'_> {
        Self {
            data: self.data,
            hash: self.hash(),
        }
    }

    /// Transforms this chunk into an [`OwnedDataChunk`]
    fn into_owned(self) -> crate::OwnedDataChunk {
        let Self { data, hash } = self;

        crate::OwnedDataChunk::from_data_and_hash(Arc::from(data), hash)
    }
}

impl<'lt, T: DataChunk> From<&'lt T> for BorrowedDataChunk<'lt> {
    fn from(chunk: &'lt T) -> Self {
        Self::from_parts(chunk.data_ref(), chunk.hash())
    }
}
