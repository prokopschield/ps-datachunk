pub mod aligned;
pub mod borrowed;
pub mod encrypted;
pub mod error;
pub mod mbuf;
pub mod owned;
pub mod serialized;
pub mod shared;
pub mod typed;
pub mod utils;
pub use aligned::AlignedDataChunk;
pub use borrowed::BorrowedDataChunk;
pub use encrypted::EncryptedDataChunk;
pub use error::PsDataChunkError;
pub use error::Result;
pub use mbuf::MbufDataChunk;
pub use owned::OwnedDataChunk;
pub use ps_hash::Hash;
pub use ps_mbuf::Mbuf;
pub use serialized::SerializedDataChunk;
pub use shared::SharedDataChunk;
pub use typed::ToDataChunk;
pub use typed::ToTypedDataChunk;
pub use typed::TypedDataChunk;

use std::sync::Arc;

/// represents any representation of a chunk of data
pub trait DataChunkTrait {
    fn data_ref(&self) -> &[u8];
    fn hash_ref(&self) -> &[u8];

    fn hash(&self) -> Arc<Hash> {
        ps_hash::hash(self.data_ref()).into()
    }

    fn encrypt(&self) -> Result<EncryptedDataChunk> {
        self.serialize()?.encrypt()
    }

    fn decrypt(&self, key: &[u8]) -> Result<SerializedDataChunk> {
        utils::decrypt(self.data_ref(), key)
    }

    fn to_datachunk(&self) -> DataChunk {
        DataChunk::Borrowed(self.borrow())
    }

    fn borrow(&self) -> BorrowedDataChunk {
        BorrowedDataChunk::from_parts(self.data_ref(), self.hash())
    }

    fn to_owned(&self) -> OwnedDataChunk {
        let data_ref = self.data_ref();
        let reserved_size = utils::offsets(data_ref.len()).2;
        let mut data_vec = Vec::with_capacity(reserved_size);

        data_vec.extend_from_slice(data_ref);

        OwnedDataChunk::from_parts(data_vec, self.hash())
    }

    fn serialize(&self) -> Result<SerializedDataChunk> {
        SerializedDataChunk::from_parts(self.data_ref(), self.hash())
    }

    fn try_as<T: rkyv::Archive>(&self) -> Result<TypedDataChunk<T>>
    where
        T::Archived:
            for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rancor::Error>>,
    {
        match typed::check_byte_layout::<T>(self.data_ref()) {
            true => Ok(unsafe { TypedDataChunk::from_chunk_unchecked(self.to_datachunk()) }),
            false => Err(PsDataChunkError::TypeError),
        }
    }
}

/// represents a chunk of data that is either owned or pointed to
pub enum DataChunk<'lt> {
    Aligned(AlignedDataChunk),
    Borrowed(BorrowedDataChunk<'lt>),
    Mbuf(MbufDataChunk<'lt>),
    Owned(OwnedDataChunk),
    Serialized(SerializedDataChunk),
    Shared(SharedDataChunk),
}

impl<'lt> DataChunkTrait for DataChunk<'lt> {
    fn data_ref(&self) -> &[u8] {
        self.data_ref()
    }
    fn hash_ref(&self) -> &[u8] {
        self.hash_ref()
    }
    fn hash(&self) -> Arc<Hash> {
        self.hash()
    }
}

impl<'lt> From<AlignedDataChunk> for DataChunk<'lt> {
    fn from(chunk: AlignedDataChunk) -> Self {
        Self::Aligned(chunk)
    }
}

impl<'lt> From<BorrowedDataChunk<'lt>> for DataChunk<'lt> {
    fn from(chunk: BorrowedDataChunk<'lt>) -> Self {
        Self::Borrowed(chunk)
    }
}

impl<'lt> From<MbufDataChunk<'lt>> for DataChunk<'lt> {
    fn from(chunk: MbufDataChunk<'lt>) -> Self {
        Self::Mbuf(chunk)
    }
}

impl<'lt> From<OwnedDataChunk> for DataChunk<'lt> {
    fn from(chunk: OwnedDataChunk) -> Self {
        Self::Owned(chunk)
    }
}

impl<'lt> DataChunk<'lt> {
    pub fn data_ref(&self) -> &[u8] {
        match self {
            Self::Borrowed(borrowed) => borrowed.data_ref(),
            Self::Aligned(aligned) => aligned.data_ref(),
            Self::Mbuf(mbuf) => mbuf.data_ref(),
            Self::Owned(owned) => owned.data_ref(),
            Self::Serialized(serialized) => serialized.data_ref(),
            Self::Shared(shared) => shared.data_ref(),
        }
    }

    pub fn hash_ref(&self) -> &[u8] {
        match self {
            Self::Aligned(aligned) => aligned.hash_ref(),
            Self::Borrowed(borrowed) => borrowed.hash_ref(),
            Self::Mbuf(mbuf) => mbuf.hash_ref(),
            Self::Owned(owned) => owned.hash_ref(),
            Self::Serialized(serialized) => serialized.hash_ref(),
            Self::Shared(shared) => shared.hash_ref(),
        }
    }

    pub fn hash(&self) -> Arc<Hash> {
        match self {
            Self::Aligned(aligned) => aligned.hash(),
            Self::Borrowed(borrowed) => borrowed.hash(),
            Self::Mbuf(mbuf) => mbuf.hash(),
            Self::Owned(owned) => owned.hash(),
            Self::Serialized(serialized) => serialized.hash(),
            Self::Shared(shared) => shared.hash(),
        }
    }

    #[inline(always)]
    /// Decrypts this [DataChunk] with a given key.
    pub fn decrypt(&self, key: &[u8]) -> Result<SerializedDataChunk> {
        let decrypted = match self {
            Self::Borrowed(borrowed) => borrowed.decrypt(key),
            Self::Mbuf(mbuf) => mbuf.decrypt(key),
            Self::Owned(chunk) => chunk.decrypt(key),
            Self::Aligned(aligned) => aligned.decrypt(key),
            Self::Serialized(serialized) => serialized.decrypt(key),
            Self::Shared(shared) => shared.decrypt(key),
        }?;

        Ok(decrypted)
    }

    #[inline(always)]
    /// Encrypts this [DataChunk].
    pub fn encrypt(&self) -> Result<EncryptedDataChunk> {
        match self {
            DataChunk::Owned(owned) => owned.encrypt(),
            DataChunk::Aligned(aligned) => {
                OwnedDataChunk::encrypt_serialized_bytes(aligned.serialize()?.serialized_bytes())
            }
            DataChunk::Serialized(serialized) => serialized.encrypt(),
            DataChunk::Shared(shared) => shared.encrypt(),
            _ => self.serialize()?.encrypt(),
        }
    }

    #[inline(always)]
    /// Encrypts this [DataChunk] using `self.data` if owned.
    pub fn encrypt_mut(&mut self) -> Result<EncryptedDataChunk> {
        match self {
            DataChunk::Owned(chunk) => chunk.encrypt_mut(),
            _ => self.encrypt(),
        }
    }

    pub fn guarantee_alignment<T>(self) -> Result<DataChunk<'lt>> {
        let align_size = std::mem::align_of::<T>();
        let remainder = self.data_ref().as_ptr() as usize % align_size;

        let chunk = if remainder == 0 {
            self
        } else {
            DataChunk::Serialized(self.serialize()?)
        };

        Ok(chunk)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() -> Result<()> {
        let original_data = "Neboť tak Bůh miluje svět, že dal [svého] jediného Syna, aby žádný, kdo v něho věří, nezahynul, ale měl život věčný. Vždyť Bůh neposlal [svého] Syna na svět, aby svět odsoudil, ale aby byl svět skrze něj zachráněn.".as_bytes().to_owned();

        let data_chunk = DataChunk::Owned(OwnedDataChunk::from_data_ref(&original_data));

        let encrypted_chunk = data_chunk.encrypt()?;
        let decrypted_chunk = encrypted_chunk.decrypt()?;

        assert_eq!(decrypted_chunk.data_ref(), original_data);

        Ok(())
    }

    #[test]
    fn test_serialization() -> Result<()> {
        let original_data = vec![1, 2, 3, 4, 5];
        let hash = ps_hash::hash(&original_data).into();
        let owned_chunk = OwnedDataChunk::from_parts(original_data.to_vec(), hash);
        let data_chunk = DataChunk::Owned(owned_chunk);

        let serialized = data_chunk.serialize()?;
        let deserialized = OwnedDataChunk::deserialize(&serialized.into_buffer())?;

        assert_eq!(deserialized.data_ref(), original_data);

        Ok(())
    }
}
