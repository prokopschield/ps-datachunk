use crate::DataChunk;
use crate::PsDataChunkError;
use rkyv::validation::validators::DefaultValidator;
use rkyv::Archive;
use rkyv::CheckBytes;
use std::marker::PhantomData;
use std::ops::Deref;

pub struct TypedDataChunk<'lt, T: rkyv::Archive> {
    chunk: DataChunk<'lt>,
    _p: PhantomData<T::Archived>,
}

pub fn check_byte_layout<'lt, T>(bytes: &[u8]) -> bool
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<DefaultValidator<'a>>,
{
    rkyv::check_archived_root::<T>(bytes).is_ok()
}

impl<'lt, T> TryFrom<DataChunk<'lt>> for TypedDataChunk<'lt, T>
where
    T: Archive,
    T::Archived: for<'a> CheckBytes<DefaultValidator<'a>>,
{
    type Error = PsDataChunkError;

    fn try_from(chunk: DataChunk<'lt>) -> Result<TypedDataChunk<'lt, T>, Self::Error> {
        match check_byte_layout::<T>(chunk.data()) {
            true => Ok(TypedDataChunk {
                chunk,
                _p: PhantomData,
            }),
            false => Err(PsDataChunkError::TypeError),
        }
    }
}

impl<'lt, T: Archive> Deref for TypedDataChunk<'lt, T>
where
    <T as Archive>::Archived: CheckBytes<DefaultValidator<'lt>>,
{
    type Target = T::Archived;

    fn deref(&self) -> &Self::Target {
        unsafe { rkyv::archived_root::<T>(self.chunk.data()) }
    }
}
