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
    T::Archived: CheckBytes<DefaultValidator<'lt>> + 'lt,
{
    let fake: &'lt [u8] = unsafe { std::slice::from_raw_parts(bytes.as_ptr(), bytes.len()) };

    rkyv::check_archived_root::<'lt, T>(fake).is_ok()
}

impl<'lt, T> TryFrom<DataChunk<'lt>> for TypedDataChunk<'lt, T>
where
    T: Archive,
    T::Archived: CheckBytes<DefaultValidator<'lt>> + 'lt,
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
