use std::{marker::PhantomData, ops::Deref};

use rancor::{Error, Strategy};
use rkyv::{
    api::high::HighValidator,
    bytecheck::CheckBytes,
    ser::{allocator::ArenaHandle, sharing::Share, Serializer},
    util::AlignedVec,
    Archive, Serialize,
};

use crate::*;

pub struct TypedDataChunk<'lt, T: rkyv::Archive> {
    chunk: DataChunk<'lt>,
    _p: PhantomData<T::Archived>,
}

pub fn check_byte_layout<'lt, T>(bytes: &[u8]) -> bool
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<HighValidator<'a, Error>>,
{
    rkyv::access::<T::Archived, Error>(bytes).is_ok()
}

impl<'lt, T: Archive> TypedDataChunk<'lt, T> {
    pub unsafe fn from_chunk_unchecked(chunk: DataChunk<'lt>) -> Self {
        Self {
            chunk,
            _p: PhantomData,
        }
    }
}

impl<'lt, T> TryFrom<DataChunk<'lt>> for TypedDataChunk<'lt, T>
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<HighValidator<'a, Error>>,
{
    type Error = PsDataChunkError;

    fn try_from(chunk: DataChunk<'lt>) -> Result<TypedDataChunk<'lt, T>> {
        match check_byte_layout::<T>(chunk.data_ref()) {
            true => Ok(unsafe { TypedDataChunk::from_chunk_unchecked(chunk) }),
            false => Err(PsDataChunkError::TypeError),
        }
    }
}

impl<'lt, T: Archive> Deref for TypedDataChunk<'lt, T>
where
    <T as Archive>::Archived: CheckBytes<HighValidator<'lt, Error>>,
{
    type Target = T::Archived;

    fn deref(&self) -> &Self::Target {
        unsafe { rkyv::access_unchecked::<T::Archived>(self.chunk.data_ref()) }
    }
}

impl<'lt, T: Archive> DataChunkTrait for TypedDataChunk<'lt, T> {
    fn data_ref(&self) -> &[u8] {
        self.chunk.data_ref()
    }

    fn hash_ref(&self) -> &[u8] {
        self.chunk.hash_ref()
    }

    fn hash(&self) -> Arc<Hash> {
        self.chunk.hash()
    }
}

pub trait ToDataChunk {
    fn to_datachunk(&self) -> Result<DataChunk>;
}

impl<T: Archive + ToTypedDataChunk<T>> ToDataChunk for T {
    fn to_datachunk(&self) -> Result<DataChunk> {
        Ok(self.to_typed_datachunk()?.chunk)
    }
}

pub unsafe trait ToTypedDataChunk<T: Archive> {
    fn to_typed_datachunk(&self) -> Result<TypedDataChunk<'static, T>>;
}

unsafe impl<T> ToTypedDataChunk<T> for T
where
    T: rkyv::Archive,
    T::Archived: for<'a> CheckBytes<HighValidator<'a, Error>>,
    T: for<'a> Serialize<Strategy<Serializer<AlignedVec, ArenaHandle<'a>, Share>, Error>>,
{
    fn to_typed_datachunk(&self) -> Result<TypedDataChunk<'static, T>> {
        let aligned = AlignedDataChunk::try_from::<T>(self)?;
        let chunk = DataChunk::Aligned(aligned);
        let typed = unsafe { TypedDataChunk::from_chunk_unchecked(chunk) };

        Ok(typed)
    }
}
