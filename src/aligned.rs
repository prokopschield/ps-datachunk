use std::ops::Deref;

use ps_hash::hash;
use rancor::{Error, Strategy};
use rkyv::{
    api::high::HighSerializer,
    bytecheck::CheckBytes,
    ser::allocator::ArenaHandle,
    util::AlignedVec,
    validation::{archive::ArchiveValidator, shared::SharedValidator, Validator},
    Archive, Serialize,
};

use crate::{Arc, DataChunk, Hash, Result};

#[derive(Debug, Clone)]
pub struct AlignedDataChunk {
    data: AlignedVec,
    hash: Arc<Hash>,
}

impl AlignedDataChunk {
    pub fn from_parts<D, H>(data: D, hash: H) -> Self
    where
        D: Into<AlignedVec>,
        H: Into<Arc<Hash>>,
    {
        let data = data.into();
        let hash = hash.into();

        Self { data, hash }
    }

    pub fn from_data_vec(data: AlignedVec) -> Result<Self> {
        let hash = hash(&data)?;

        Ok(Self::from_parts(data, hash))
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl AsRef<[u8]> for AlignedDataChunk {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

impl Deref for AlignedDataChunk {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl AlignedDataChunk {
    pub fn try_from<T>(value: &T) -> Result<Self>
    where
        for<'a> T: Archive + Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, Error>>,
    {
        let data = rkyv::to_bytes::<Error>(value)
            .map_err(|err| crate::PsDataChunkError::RkyvSerializationFailed(err.into()))?;

        Self::from_data_vec(data)
    }

    pub fn try_bytes_as<T: rkyv::Archive>(data: &[u8]) -> Result<&T::Archived>
    where
        for<'a> <T as rkyv::Archive>::Archived:
            CheckBytes<Strategy<Validator<ArchiveValidator<'a>, SharedValidator>, rancor::Error>>,
    {
        rkyv::access::<T::Archived, Error>(data)
            .map_err(|err| crate::PsDataChunkError::RkyvInvalidArchive(err.into()))
    }

    pub fn try_as<T: rkyv::Archive>(&self) -> Result<&T::Archived>
    where
        for<'a> <T as Archive>::Archived:
            CheckBytes<Strategy<Validator<ArchiveValidator<'a>, SharedValidator>, rancor::Error>>,
    {
        Self::try_bytes_as::<T>(self.data_ref())
    }
}

impl DataChunk for AlignedDataChunk {
    fn data_ref(&self) -> &[u8] {
        self
    }
    fn hash_ref(&self) -> &Hash {
        &self.hash
    }
    fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }
}
