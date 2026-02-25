mod implementations;

use crate::DataChunk;
use crate::Result;
use bytes::Bytes;
use ps_hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// represents an owned chunk of data
pub struct OwnedDataChunk {
    hash: Hash,
    data: Bytes,
}

impl OwnedDataChunk {
    /// Returns this chunk's bytes.
    ///
    /// This is a cheap clone of the underlying `Bytes` buffer.
    #[must_use]
    pub fn bytes(&self) -> Bytes {
        self.data.clone()
    }

    #[must_use]
    pub fn data_ref(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub const fn hash_ref(&self) -> &Hash {
        &self.hash
    }

    #[must_use]
    pub const fn hash(&self) -> Hash {
        self.hash
    }

    /// Creates an [`OwnedDataChunk`] from its constituent parts.
    /// # Safety
    /// - `hash` must be the hash of `data`
    /// - use `from_data()` if you cannot ensure this
    #[inline]
    #[must_use]
    pub const fn from_parts_unchecked(data: Bytes, hash: Hash) -> Self {
        Self { hash, data }
    }

    pub fn from_data_and_hash_unchecked<D>(data: D, hash: Hash) -> Self
    where
        D: AsRef<[u8]> + Send + 'static,
    {
        Self::from_parts_unchecked(Bytes::from_owner(data), hash)
    }

    /// calculates the hash of `data` and returns an `OwnedDataChunk`
    pub fn from_bytes(data: Bytes) -> Result<Self> {
        let hash = ps_hash::hash(&data)?;

        Ok(Self::from_parts_unchecked(data, hash))
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

    /// Transforms this [`DataChunk`] into [`Bytes`].
    fn into_bytes(self) -> Bytes {
        self.data
    }

    /// Transforms this chunk into an [`OwnedDataChunk`]
    fn into_owned(self) -> Self {
        self
    }
}
