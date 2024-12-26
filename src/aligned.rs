use crate::*;
use ps_hash::{hash, Hash};
use rkyv::AlignedVec;
use std::{ops::Deref, sync::Arc};

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
    pub fn try_from<
        const S: usize,
        T: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<S>>,
    >(
        value: &T,
    ) -> Result<Self> {
        let data = rkyv::to_bytes(value).map_err(|_| PsDataChunkError::SerializationError)?;
        let chunk = Self::from_data_vec(data);

        Ok(chunk)
    }

    pub fn try_bytes_as<'lt, T: rkyv::Archive>(data: &'lt [u8]) -> Result<&'lt T::Archived>
    where
        <T as rkyv::Archive>::Archived:
            rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'lt>>,
    {
        let result = rkyv::check_archived_root::<T>(data);
        let value = result.map_err(|_| PsDataChunkError::TypeError)?;

        Ok(value)
    }

    pub fn try_as<'lt, T: rkyv::Archive>(&'lt self) -> Result<&'lt T::Archived>
    where
        <T as rkyv::Archive>::Archived:
            rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'lt>>,
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
    pub fn try_from<
        const S: usize,
        T: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<S>>,
    >(
        value: &T,
    ) -> Result<Self> {
        Ok(Self::Aligned(AlignedDataChunk::try_from(value)?))
    }

    pub fn try_as<T: rkyv::Archive>(&'lt self) -> Result<&'lt T::Archived>
    where
        <T as rkyv::Archive>::Archived:
            rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'lt>>,
    {
        match self {
            Self::Aligned(aligned) => aligned.try_as::<T>(),
            Self::Borrowed(borrowed) => AlignedDataChunk::try_bytes_as::<T>(borrowed.data_ref()),
            Self::Mbuf(mbuf) => AlignedDataChunk::try_bytes_as::<T>(mbuf.data_ref()),
            Self::Owned(owned) => AlignedDataChunk::try_bytes_as::<T>(owned.data_ref()),
            Self::Shared(shared) => AlignedDataChunk::try_bytes_as::<T>(shared.data_ref()),
        }
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
            let chunk = AlignedDataChunk::try_from::<4, _>(&data)?;

            assert_eq!(chunk.serialize().serialized_bytes().len() % 16, 0);

            let (hash_offset, size_offset, size) = offsets(i);

            assert_eq!(hash_offset % 16, 0);
            assert_eq!(size_offset % 8, 0);
            assert_eq!(size % 16, 0);
        }

        Ok(())
    }
}
