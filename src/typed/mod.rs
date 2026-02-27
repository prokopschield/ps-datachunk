mod implementations;

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
    /// Builds a typed view after validating that the byte layout is a valid archived `T`.
    ///
    /// This method assumes `D` upholds [`crate::DataChunk`] invariants (stable, immutable
    /// bytes/hash for `&self`) for the lifetime of this value.
    pub fn from_data_chunk(chunk: D) -> Result<Self> {
        rkyv::access::<T::Archived, Error>(chunk.data_ref())
            .map_err(|_| crate::DataChunkError::InvalidArchive)?;

        let chunk = Self {
            _p: PhantomData,
            chunk,
        };

        Ok(chunk)
    }

    /// Returns a checked typed reference.
    ///
    /// Unlike [`Deref`], this method always validates the underlying bytes before
    /// returning the archived value.
    pub fn typed_ref(&self) -> Result<&T::Archived> {
        rkyv::access::<T::Archived, Error>(self.chunk.data_ref())
            .map_err(|_| crate::DataChunkError::InvalidArchive)
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
        // SAFETY:
        // - `from_data_chunk` validates that `chunk.data_ref()` contains a valid `T::Archived`.
        // - `TypedDataChunk` only exposes shared access to `chunk`, so no mutation happens
        //   through this type after validation.
        // - This relies on the `DataChunk` contract that bytes/hash are stable and immutable for `&self`.
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

#[allow(clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataChunkError, OwnedDataChunk};

    #[test]
    fn typed_ref_returns_checked_ref() -> Result<()> {
        let typed = 42_u32.to_typed_datachunk()?;

        assert_eq!(*typed.typed_ref()?, 42_u32);
        assert_eq!(*typed, *typed.typed_ref()?);

        Ok(())
    }

    #[test]
    fn from_data_chunk_rejects_invalid_archive() {
        let chunk = OwnedDataChunk::from_data([1_u8, 2, 3]).expect("hashing failed");

        let result = TypedDataChunk::<OwnedDataChunk, u32>::from_data_chunk(chunk);

        assert!(matches!(result, Err(DataChunkError::InvalidArchive)));
    }
}
