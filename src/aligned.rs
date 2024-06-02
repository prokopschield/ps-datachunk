use crate::*;
use ps_hash::Hash;
use rkyv::AlignedVec;

#[derive(rkyv::Archive, rkyv::Serialize, Debug, Clone)]
pub struct AlignedDataChunk {
    inner: AlignedVec,
}

pub const fn rup(size: usize, n: usize) -> usize {
    (((size - 1) >> n) + 1) << n
}

pub const fn rdown(size: usize, n: usize) -> usize {
    (size >> n) << n
}

pub const HSIZE: usize = rup(std::mem::size_of::<Hash>(), 3);
pub const SSIZE: usize = rup(std::mem::size_of::<usize>(), 3);

pub const fn offsets(dsize: usize, hsize: usize) -> (usize, usize, usize) {
    let hash_offset = rup(dsize, 4);
    let size_offset = rup(hash_offset + hsize, 3);
    let size = rup(size_offset + SSIZE, 4);

    (hash_offset, size_offset, size)
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
        let (hash_offset, size_offset, size) = offsets(chunk_data.len(), hash.len());

        let mut data = AlignedVec::with_capacity(size);

        data.extend_from_slice(chunk_data);

        Self::new_with_parameters(data, hash, chunk_data.len(), hash_offset, size_offset)
    }

    pub fn new_from_data_vec(data: AlignedVec) -> Self {
        let (hash_offset, size_offset, _) = offsets(data.len(), HSIZE);
        let hash = ps_hash::hash(data.as_slice());
        let data_length = data.len();

        Self::new_with_parameters(data, hash.as_bytes(), data_length, hash_offset, size_offset)
    }

    pub fn new(chunk_data: &[u8]) -> Self {
        Self::new_with_hash(chunk_data, ps_hash::hash(chunk_data).as_bytes())
    }

    pub fn len(&self) -> usize {
        let begin = rdown(self.inner.len() - 1, 3);
        let end = begin + std::mem::size_of::<usize>();
        let range = begin..end;

        if let Ok(bytes) = self.inner[range].try_into() {
            usize::from_le_bytes(bytes)
        } else {
            0
        }
    }

    pub fn hash_ref(&self) -> &[u8] {
        let begin = rdown(self.inner.len() - HSIZE, 4);
        let end = begin + std::mem::size_of::<Hash>();
        let range = begin..end;

        &self.inner[range]
    }

    pub fn hash(&self) -> [u8; 50] {
        if let Ok(hash) = &self.hash_ref().try_into() {
            *hash
        } else {
            [0u8; 50]
        }
    }

    pub fn data(&self) -> &[u8] {
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

    pub fn deserialize(bytes: &[u8]) -> Result<Self, PsDataChunkError> {
        if bytes.len() < HSIZE + SSIZE {
            return Err(PsDataChunkError::InvalidDataChunk);
        }

        let chunk = Self::deserialize_unchecked(bytes);

        if ps_hash::hash(chunk.data()).as_bytes() != chunk.hash_ref() {
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
        Self::new_with_hash(&value.data, &value.hash)
    }
}

impl<'lt> From<&DataChunk<'lt>> for AlignedDataChunk {
    fn from(value: &DataChunk) -> Self {
        match value {
            DataChunk::Mbuf(mbuf) => AlignedDataChunk::new_with_hash(*mbuf, mbuf.get_metadata()),
            DataChunk::Owned(owned) => owned.into(),
            DataChunk::Aligned(aligned) => aligned.clone(),
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

impl Into<OwnedDataChunk> for AlignedDataChunk {
    fn into(self) -> OwnedDataChunk {
        OwnedDataChunk {
            hash: self.hash(),
            data: self.data().to_vec(),
        }
    }
}

impl<'lt> Into<DataChunk<'lt>> for AlignedDataChunk {
    fn into(self) -> DataChunk<'lt> {
        DataChunk::Aligned(self)
    }
}

impl AlignedDataChunk {
    pub fn try_from<
        const S: usize,
        T: rkyv::Archive + rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<S>>,
    >(
        value: &T,
    ) -> Result<Self, PsDataChunkError> {
        let data = rkyv::to_bytes(value).map_err(|_| PsDataChunkError::SerializationError)?;
        let chunk = Self::new_from_data_vec(data);

        Ok(chunk)
    }

    pub fn try_as<'lt, T: rkyv::Archive>(&'lt self) -> Result<&'lt T::Archived, PsDataChunkError>
    where
        <T as rkyv::Archive>::Archived:
            rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'lt>>,
    {
        let result = rkyv::check_archived_root::<T>(self.data());
        let value = result.map_err(|_| PsDataChunkError::DeserializationError)?;

        Ok(value)
    }
}
