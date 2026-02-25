use std::{marker::PhantomData, ops::Deref};

use bytes::Bytes;
use rancor::{Error, Strategy};
use rkyv::{
    api::high::HighValidator,
    bytecheck::CheckBytes,
    ser::{allocator::ArenaHandle, sharing::Share, Serializer},
    util::AlignedVec,
    Archive, Serialize,
};

use crate::{AlignedDataChunk, DataChunk, Hash, Result};

pub struct TypedDataChunk<D: DataChunk, T: rkyv::Archive> {
    chunk: D,
    _p: PhantomData<T::Archived>,
}

#[must_use]
pub fn check_byte_layout<'lt, T>(bytes: &[u8]) -> bool
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<HighValidator<'a, Error>>,
{
    rkyv::access::<T::Archived, Error>(bytes).is_ok()
}

impl<D, T> TypedDataChunk<D, T>
where
    D: DataChunk,
    T: Archive,
    T::Archived: for<'a> CheckBytes<HighValidator<'a, Error>>,
{
    pub fn from_data_chunk(chunk: D) -> Result<Self> {
        rkyv::access::<T::Archived, Error>(chunk.data_ref())
            .map_err(|_| crate::PsDataChunkError::RkyvInvalidArchive)?;

        let chunk = Self {
            _p: PhantomData,
            chunk,
        };

        Ok(chunk)
    }
}

impl<D, T> Deref for TypedDataChunk<D, T>
where
    D: DataChunk,
    T: Archive,
    for<'a> <T as Archive>::Archived: CheckBytes<HighValidator<'a, Error>>,
{
    type Target = T::Archived;

    fn deref(&self) -> &Self::Target {
        unsafe { rkyv::access_unchecked::<T::Archived>(self.chunk.data_ref()) }
    }
}

impl<D, T> DataChunk for TypedDataChunk<D, T>
where
    D: DataChunk,
    T: Archive,
    for<'a> <T as Archive>::Archived: CheckBytes<HighValidator<'a, Error>>,
{
    fn data_ref(&self) -> &[u8] {
        self.chunk.data_ref()
    }

    fn hash_ref(&self) -> &Hash {
        self.chunk.hash_ref()
    }

    /// Transforms this [`DataChunk`] into [`Bytes`].
    fn into_bytes(self) -> Bytes {
        self.chunk.into_bytes()
    }

    /// Transforms this chunk into an [`OwnedDataChunk`]
    fn into_owned(self) -> crate::OwnedDataChunk {
        let Self { chunk, _p } = self;

        chunk.into_owned()
    }
}

pub trait ToDataChunk {
    fn to_datachunk(&self) -> Result<AlignedDataChunk>;
}

impl<T: Archive + ToTypedDataChunk<T>> ToDataChunk for T {
    fn to_datachunk(&self) -> Result<AlignedDataChunk> {
        Ok(self.to_typed_datachunk()?.chunk)
    }
}

pub trait ToTypedDataChunk<T: Archive> {
    fn to_typed_datachunk(&self) -> Result<TypedDataChunk<AlignedDataChunk, T>>;
}

impl<T> ToTypedDataChunk<T> for T
where
    T::Archived: for<'a> CheckBytes<HighValidator<'a, Error>>,
    T: rkyv::Archive
        + for<'a> Serialize<Strategy<Serializer<AlignedVec, ArenaHandle<'a>, Share>, Error>>,
{
    fn to_typed_datachunk(&self) -> Result<TypedDataChunk<AlignedDataChunk, T>> {
        let chunk = AlignedDataChunk::try_from::<T>(self)?;

        TypedDataChunk::from_data_chunk(chunk)
    }
}
