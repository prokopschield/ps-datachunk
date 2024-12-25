use crate::error::Result;
use crate::AlignedDataChunk;
use crate::DataChunk;
use crate::DataChunkTrait;
use crate::PsDataChunkError;
use ps_hash::Hash;
use rkyv::validation::validators::DefaultValidator;
use rkyv::Archive;
use rkyv::CheckBytes;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

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
    T::Archived: for<'a> CheckBytes<DefaultValidator<'a>>,
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
    <T as Archive>::Archived: CheckBytes<DefaultValidator<'lt>>,
{
    type Target = T::Archived;

    fn deref(&self) -> &Self::Target {
        unsafe { rkyv::archived_root::<T>(self.chunk.data_ref()) }
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
    T::Archived: for<'a> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
    T::Archived: rkyv::Serialize<rkyv::ser::serializers::AllocSerializer<64>>,
    T: rkyv::Serialize<
        rkyv::ser::serializers::CompositeSerializer<
            rkyv::ser::serializers::AlignedSerializer<rkyv::AlignedVec>,
            rkyv::ser::serializers::FallbackScratch<
                rkyv::ser::serializers::HeapScratch<64>,
                rkyv::ser::serializers::AllocScratch,
            >,
            rkyv::ser::serializers::SharedSerializeMap,
        >,
    >,
{
    fn to_typed_datachunk(&self) -> Result<TypedDataChunk<'static, T>> {
        let aligned = AlignedDataChunk::try_from::<64, T>(self)?;
        let chunk = DataChunk::Aligned(aligned);
        let typed = unsafe { TypedDataChunk::from_chunk_unchecked(chunk) };

        Ok(typed)
    }
}
