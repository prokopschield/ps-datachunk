use crate::DataChunk;
use crate::Result;
use bytes::Bytes;
use ps_hash::Hash;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// represents an owned chunk of data
pub struct OwnedDataChunk {
    hash: Arc<Hash>,
    data: Bytes,
}

impl OwnedDataChunk {
    #[must_use]
    pub fn data_ref(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub fn hash_ref(&self) -> &Hash {
        &self.hash
    }

    #[must_use]
    pub fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }

    /// Creates an [`OwnedDataChunk`] from its constituent parts
    /// # Safety
    /// - `hash` must be the hash of `data`
    /// - use `from_data()` if you cannot ensure this
    #[inline]
    #[must_use]
    pub const fn from_parts(data: Bytes, hash: Arc<Hash>) -> Self {
        Self { hash, data }
    }

    pub fn from_data_and_hash<D>(data: D, hash: Arc<Hash>) -> Self
    where
        D: AsRef<[u8]> + Send + 'static,
    {
        Self::from_parts(Bytes::from_owner(data), hash)
    }

    /// calculates the hash of `data` and returns an `OwnedDataChunk`
    pub fn from_bytes(data: Bytes) -> Result<Self> {
        let hash = ps_hash::hash(&data)?;

        Ok(Self::from_parts(data, hash.into()))
    }

    /// calculates the hash of `data` and returns an `OwnedDataChunk`
    pub fn from_data<D>(data: D) -> Result<Self>
    where
        D: AsRef<[u8]> + Send + 'static,
    {
        Self::from_bytes(Bytes::from_owner(data))
    }
}

impl DataChunk for OwnedDataChunk {
    fn data_ref(&self) -> &[u8] {
        self.data_ref()
    }
    fn hash_ref(&self) -> &Hash {
        self.hash_ref()
    }
    fn hash(&self) -> Arc<Hash> {
        self.hash()
    }

    /// Transforms this [`DataChunk`] into [`Bytes`].
    fn into_bytes(self) -> Bytes {
        self.data
    }

    /// Transforms this chunk into an [`OwnedDataChunk`]
    fn into_owned(self) -> Self {
        self
    }
}
