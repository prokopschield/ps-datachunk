use crate::*;
use ps_hash::Hash;
use rkyv::AlignedVec;
use std::{borrow::Cow, sync::Arc};
use utils::{
    constants::{HASH_ALIGNMENT, HASH_SIZE, SIZE_ALIGNMENT, SIZE_SIZE},
    offsets::offsets,
    rounding::round_down,
};

#[derive(rkyv::Archive, rkyv::Serialize, Debug, Clone)]
pub struct AlignedDataChunk {
    inner: AlignedVec,
}

impl AlignedDataChunk {
    pub fn new_with_parameters(
        mut data: AlignedVec,
        hash: &[u8],
        data_length: usize,
        hash_offset: usize,
        size_offset: usize,
    ) -> Self {
        data.extend_from_slice(&vec![0u8; hash_offset - data.len()]);
        data.extend_from_slice(hash);
        data.extend_from_slice(&vec![0u8; size_offset - data.len()]);
        data.extend_from_slice(&data_length.to_le_bytes());

        Self { inner: data }
    }

    pub fn new_with_hash(chunk_data: &[u8], hash: &[u8]) -> Self {
        let (hash_offset, size_offset, size) = offsets(chunk_data.len());

        let mut data = AlignedVec::with_capacity(size);

        data.extend_from_slice(chunk_data);

        Self::new_with_parameters(data, hash, chunk_data.len(), hash_offset, size_offset)
    }

    pub fn new_from_data_vec(data: AlignedVec) -> Self {
        let (hash_offset, size_offset, _) = offsets(data.len());
        let hash = ps_hash::hash(data.as_slice());
        let data_length = data.len();

        Self::new_with_parameters(data, hash.as_bytes(), data_length, hash_offset, size_offset)
    }

    pub fn new(chunk_data: &[u8]) -> Self {
        Self::new_with_hash(chunk_data, ps_hash::hash(chunk_data).as_bytes())
    }

    pub fn len(&self) -> usize {
        let begin = round_down(self.inner.len() - 1, SIZE_ALIGNMENT);
        let end = begin + std::mem::size_of::<usize>();
        let range = begin..end;

        if let Ok(bytes) = self.inner[range].try_into() {
            usize::from_le_bytes(bytes)
        } else {
            0
        }
    }

    pub fn hash_ref(&self) -> &[u8] {
        let begin = round_down(self.inner.len() - HASH_SIZE, HASH_ALIGNMENT);
        let end = begin + std::mem::size_of::<Hash>();
        let range = begin..end;

        &self.inner[range]
    }

    pub fn hash(&self) -> Hash {
        match Hash::try_from(self.hash_ref()) {
            Ok(hash) => hash,
            Err(_) => ps_hash::hash(self.data_ref()),
        }
    }

    pub fn data_ref(&self) -> &[u8] {
        &self.inner[0..self.len()]
    }

    pub fn serialize_into(self) -> AlignedVec {
        self.inner
    }

    pub fn serialize(&self) -> AlignedVec {
        self.inner.clone()
    }

    pub fn deserialize_from(inner: AlignedVec) -> Self {
        Self { inner }
    }

    pub fn deserialize_unchecked(bytes: &[u8]) -> Self {
        let mut inner = AlignedVec::with_capacity(bytes.len());

        inner.copy_from_slice(bytes);

        Self { inner }
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < HASH_SIZE + SIZE_SIZE {
            return Err(PsDataChunkError::InvalidDataChunk);
        }

        let chunk = Self::deserialize_unchecked(bytes);

        if ps_hash::hash(chunk.data_ref()).as_bytes() != chunk.hash_ref() {
            return Err(PsDataChunkError::InvalidChecksum);
        }

        Ok(chunk)
    }

    pub fn as_serialized_bytes(&self) -> &[u8] {
        &self.inner
    }
}

impl<'lt> From<&'lt Mbuf<'lt, [u8; 50], u8>> for AlignedDataChunk {
    fn from(value: &'lt Mbuf<'lt, [u8; 50], u8>) -> Self {
        Self::new_with_hash(value, value.get_metadata())
    }
}

impl From<&OwnedDataChunk> for AlignedDataChunk {
    fn from(value: &OwnedDataChunk) -> Self {
        Self::new_with_hash(value.data_ref(), value.hash_ref())
    }
}

impl<'lt> From<&DataChunk<'lt>> for AlignedDataChunk {
    fn from(value: &DataChunk) -> Self {
        match value {
            DataChunk::Borrowed(borrowed) => {
                AlignedDataChunk::new_with_hash(borrowed.data_ref(), borrowed.hash_ref())
            }
            DataChunk::Mbuf(mbuf) => {
                AlignedDataChunk::new_with_hash(mbuf.data_ref(), mbuf.hash_ref())
            }
            DataChunk::Owned(owned) => owned.into(),
            DataChunk::Aligned(aligned) => aligned.clone(),
            DataChunk::Shared(shared) => shared.align(),
        }
    }
}

impl<'lt> From<DataChunk<'lt>> for AlignedDataChunk {
    fn from(value: DataChunk) -> Self {
        match value {
            DataChunk::Aligned(aligned) => aligned,
            _ => (&value).into(),
        }
    }
}

impl Into<OwnedDataChunk> for &AlignedDataChunk {
    fn into(self) -> OwnedDataChunk {
        match OwnedDataChunk::deserialize(&self.inner) {
            Ok(owned) => owned,
            Err(_) => OwnedDataChunk::from_data_ref(self.data_ref()),
        }
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
        let chunk = Self::new_from_data_vec(data);

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
        self.data_ref()
    }
    fn hash_ref(&self) -> &[u8] {
        self.hash_ref()
    }
    fn hash(&self) -> Arc<Hash> {
        self.hash().into()
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

    pub fn align(&self) -> Cow<AlignedDataChunk> {
        match self {
            Self::Aligned(aligned) => Cow::Borrowed(aligned),
            _ => Cow::Owned(self.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_length_divisibility_and_part_alignment() -> Result<()> {
        for i in 12..256 {
            let data = (vec![i as u8; i], ());
            let chunk = AlignedDataChunk::try_from::<4, _>(&data)?;

            assert_eq!(chunk.inner.len() % 16, 0);

            let (hash_offset, size_offset, size) = offsets(i);

            assert_eq!(hash_offset % 16, 0);
            assert_eq!(size_offset % 8, 0);
            assert_eq!(size % 16, 0);
        }

        Ok(())
    }
}
