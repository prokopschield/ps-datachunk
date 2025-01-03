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

use crate::*;

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

    pub fn from_data_vec(data: AlignedVec) -> Self {
        let hash = hash(&data);

        Self::from_parts(data, hash)
    }

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
        let data = rkyv::to_bytes::<Error>(value)?;
        let chunk = Self::from_data_vec(data);

        Ok(chunk)
    }

    pub fn try_bytes_as<T: rkyv::Archive>(data: &[u8]) -> Result<&T::Archived>
    where
        for<'a> <T as rkyv::Archive>::Archived:
            CheckBytes<Strategy<Validator<ArchiveValidator<'a>, SharedValidator>, rancor::Error>>,
    {
        Ok(rkyv::access::<T::Archived, Error>(data)?)
    }

    pub fn try_as<T: rkyv::Archive>(&self) -> Result<&T::Archived>
    where
        for<'a> <T as Archive>::Archived:
            CheckBytes<Strategy<Validator<ArchiveValidator<'a>, SharedValidator>, rancor::Error>>,
    {
        Self::try_bytes_as::<T>(self.data_ref())
    }
}

impl DataChunkTrait for AlignedDataChunk {
    fn data_ref(&self) -> &[u8] {
        self
    }
    fn hash_ref(&self) -> &[u8] {
        self.hash.as_bytes()
    }
    fn hash(&self) -> Arc<Hash> {
        self.hash.clone()
    }
}

impl<'lt> DataChunk<'lt> {
    pub fn try_from<T>(value: &T) -> Result<Self>
    where
        T: Archive + for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, Error>>,
    {
        Ok(Self::Aligned(AlignedDataChunk::try_from(value)?))
    }

    pub fn try_as<T: rkyv::Archive>(&'lt self) -> Result<&'lt T::Archived>
    where
        for<'a> <T as Archive>::Archived:
            CheckBytes<Strategy<Validator<ArchiveValidator<'a>, SharedValidator>, rancor::Error>>,
    {
        AlignedDataChunk::try_bytes_as::<T>(self.data_ref())
    }
}

#[cfg(test)]
mod tests {
    use utils::offsets::offsets;

    use super::*;

    #[test]
    fn test_chunk_length_divisibility_and_part_alignment() -> Result<()> {
        for i in 12..256 {
            let data = (vec![i as u8; i], ());
            let chunk = AlignedDataChunk::try_from::<_>(&data)?;

            assert_eq!(chunk.serialize().serialized_bytes().len() % 16, 0);

            let (hash_offset, size_offset, size) = offsets(i);

            assert_eq!(hash_offset % 16, 0);
            assert_eq!(size_offset % 8, 0);
            assert_eq!(size % 16, 0);
        }

        Ok(())
    }
}
