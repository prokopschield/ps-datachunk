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
pub use borrowed::HashCow;
pub use encrypted::EncryptedDataChunk;
pub use error::PsDataChunkError;
pub use error::Result;
pub use mbuf::MbufDataChunk;
pub use owned::OwnedDataChunk;
pub use ps_cypher::Compressor;
pub use ps_hash::Hash;
pub use ps_mbuf::Mbuf;
pub use serialized::SerializedDataChunk;
pub use shared::SharedDataChunk;
pub use typed::TypedDataChunk;

/// represents any representation of a chunk of data
pub trait DataChunkTrait {
    fn data_ref(&self) -> &[u8];
    fn hash_ref(&self) -> &[u8];

    fn hash(&self) -> HashCow {
        ps_hash::hash(self.data_ref()).into()
    }

    fn encrypt(&self, compressor: &Compressor) -> Result<EncryptedDataChunk> {
        self.serialize().encrypt(compressor)
    }

    fn decrypt(&self, key: &[u8], compressor: &Compressor) -> Result<OwnedDataChunk> {
        OwnedDataChunk::decrypt_bytes(self.data_ref(), key, compressor)
    }

    fn to_datachunk(&self) -> DataChunk {
        DataChunk::Borrowed(self.borrow())
    }

    fn align(&self) -> AlignedDataChunk {
        AlignedDataChunk::new_with_hash(self.data_ref(), self.hash_ref())
    }

    fn borrow(&self) -> BorrowedDataChunk {
        BorrowedDataChunk::from_parts(self.data_ref(), self.hash())
    }

    fn to_owned(&self) -> OwnedDataChunk {
        let data_ref = self.data_ref();
        let reserved_size = utils::offsets::offsets(data_ref.len()).2;
        let mut data_vec = Vec::with_capacity(reserved_size);

        data_vec.extend_from_slice(data_ref);

        OwnedDataChunk::from_parts(data_vec, self.hash().into())
    }

    fn serialize(&self) -> SerializedDataChunk {
        SerializedDataChunk::from_parts(self.data_ref(), self.hash_ref())
    }

    fn try_as<T: rkyv::Archive>(&self) -> Result<TypedDataChunk<T>>
    where
        T::Archived: for<'a> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'a>>,
    {
        self.to_datachunk().try_into()
    }
}

/// represents a chunk of data that is either owned or pointed to
pub enum DataChunk<'lt> {
    Aligned(AlignedDataChunk),
    Borrowed(BorrowedDataChunk<'lt>),
    Mbuf(MbufDataChunk<'lt>),
    Owned(OwnedDataChunk),
    Shared(SharedDataChunk),
}

impl<'lt> DataChunkTrait for DataChunk<'lt> {
    fn data_ref(&self) -> &[u8] {
        self.data_ref()
    }
    fn hash_ref(&self) -> &[u8] {
        self.hash_ref()
    }
    fn hash(&self) -> HashCow {
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
            Self::Shared(shared) => shared.data_ref(),
        }
    }

    pub fn hash_ref(&self) -> &[u8] {
        match self {
            Self::Aligned(aligned) => aligned.hash_ref(),
            Self::Borrowed(borrowed) => borrowed.hash_ref(),
            Self::Mbuf(mbuf) => mbuf.hash_ref(),
            Self::Owned(owned) => owned.hash_ref(),
            Self::Shared(shared) => shared.hash_ref(),
        }
    }

    pub fn hash(&self) -> HashCow {
        match self {
            Self::Aligned(aligned) => aligned.hash().into(),
            Self::Borrowed(borrowed) => borrowed.hash(),
            Self::Mbuf(mbuf) => mbuf.hash(),
            Self::Owned(owned) => owned.hash().into(),
            Self::Shared(shared) => shared.hash().into(),
        }
    }

    #[inline(always)]
    /// Unwraps this [DataChunk] into an `OwnedDataChunk`.
    /// - `DataChunk::Borrowed()` allocates a new `OwnedDataChunk`, recalculates hash
    /// - `DataChunk::Mbuf()` allocates a new `OwnedDataChunk`, recalculates hash
    /// - `DataChunk::Aligned()` indirectly invokes the OwnedDataChunk deserializer
    /// - `DataChunk::Owned()` is a no-op
    pub fn into_owned(self) -> OwnedDataChunk {
        match self {
            DataChunk::Borrowed(borrowed) => OwnedDataChunk::from_data_ref(borrowed.data_ref()),
            DataChunk::Mbuf(mbuf) => OwnedDataChunk::from_data_ref(mbuf.data_ref()),
            DataChunk::Owned(chunk) => chunk,
            DataChunk::Aligned(aligned) => (&aligned).into(),
            DataChunk::Shared(shared) => shared.to_owned(),
        }
    }

    #[inline(always)]
    /// Gets an owned copy of this [DataChunk].
    pub fn to_owned(&self) -> OwnedDataChunk {
        match self {
            DataChunk::Borrowed(borrowed) => OwnedDataChunk::from_data_ref(borrowed.data_ref()),
            DataChunk::Mbuf(mbuf) => OwnedDataChunk::from_data_ref(mbuf.data_ref()),
            DataChunk::Owned(chunk) => chunk.clone(),
            DataChunk::Aligned(aligned) => aligned.into(),
            DataChunk::Shared(shared) => shared.to_owned(),
        }
    }

    #[inline(always)]
    /// convert this [DataChunk] into a `Vec<u8>`
    /// - `DataChunk::Mbuf()` is copied via `.to_owned()`
    pub fn serialize_into(self) -> Vec<u8> {
        match self {
            Self::Borrowed(_) => self.to_owned().serialize_into(),
            Self::Mbuf(_) => self.to_owned().serialize_into(),
            Self::Owned(chunk) => chunk.serialize_into(),
            Self::Aligned(aligned) => aligned.serialize_into().to_vec(),
            Self::Shared(shared) => shared.to_owned().serialize_into(),
        }
    }

    #[inline(always)]
    /// Serializes this [DataChunk] into a new `Vec<u8>`.
    pub fn serialize(&self) -> Vec<u8> {
        self.to_owned().serialize_into()
    }

    #[inline(always)]
    /// Decrypts this [DataChunk] with a given key.
    pub fn decrypt(&self, key: &[u8], compressor: &Compressor) -> Result<OwnedDataChunk> {
        let owned = match self {
            Self::Borrowed(borrowed) => {
                OwnedDataChunk::decrypt_bytes(borrowed.data_ref(), key, compressor)
            }
            Self::Mbuf(mbuf) => OwnedDataChunk::decrypt_bytes(mbuf.data_ref(), key, compressor),
            Self::Owned(chunk) => chunk.decrypt(key, compressor),
            Self::Aligned(aligned) => {
                OwnedDataChunk::decrypt_bytes(aligned.data_ref(), key, compressor)
            }
            Self::Shared(shared) => shared.decrypt(key, compressor),
        }?;

        Ok(owned)
    }

    #[inline(always)]
    /// Encrypts this [DataChunk].
    pub fn encrypt(&self, compressor: &Compressor) -> Result<EncryptedDataChunk> {
        match self {
            DataChunk::Borrowed(_) => {
                OwnedDataChunk::encrypt_serialized_bytes(&self.serialize(), compressor)
            }
            DataChunk::Mbuf(_) => {
                OwnedDataChunk::encrypt_serialized_bytes(&self.serialize(), compressor)
            }
            DataChunk::Owned(owned) => owned.encrypt(compressor),
            DataChunk::Aligned(aligned) => {
                OwnedDataChunk::encrypt_serialized_bytes(aligned.as_serialized_bytes(), compressor)
            }
            DataChunk::Shared(shared) => shared.encrypt(compressor),
        }
    }

    #[inline(always)]
    /// Encrypts this [DataChunk] using `self.data` if owned.
    pub fn encrypt_mut(&mut self, compressor: &Compressor) -> Result<EncryptedDataChunk> {
        match self {
            DataChunk::Borrowed(_) => self.encrypt(compressor),
            DataChunk::Mbuf(_) => self.encrypt(compressor),
            DataChunk::Owned(chunk) => chunk.encrypt_mut(compressor),
            DataChunk::Aligned(_) => self.encrypt(compressor),
            DataChunk::Shared(shared) => shared.encrypt(compressor),
        }
    }

    pub fn guarantee_alignment<T>(self) -> DataChunk<'lt> {
        let align_size = std::mem::align_of::<T>();
        let remainder = self.data_ref().as_ptr() as usize % align_size;

        if remainder == 0 {
            self
        } else {
            AlignedDataChunk::from(self).into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() -> Result<()> {
        let compressor = Compressor::new();
        let original_data = "Neboť tak Bůh miluje svět, že dal [svého] jediného Syna, aby žádný, kdo v něho věří, nezahynul, ale měl život věčný. Vždyť Bůh neposlal [svého] Syna na svět, aby svět odsoudil, ale aby byl svět skrze něj zachráněn.".as_bytes().to_owned();

        let data_chunk = DataChunk::Owned(OwnedDataChunk::from_data_ref(&original_data));

        let encrypted_chunk = data_chunk.encrypt(&compressor)?;
        let decrypted_chunk = encrypted_chunk.decrypt(&compressor)?;

        assert_eq!(decrypted_chunk.data_ref(), original_data);

        Ok(())
    }

    #[test]
    fn test_serialization() -> Result<()> {
        let original_data = vec![1, 2, 3, 4, 5];
        let hash = ps_hash::hash(&original_data).into();
        let owned_chunk = OwnedDataChunk::from_parts(original_data.to_vec(), hash);
        let data_chunk = DataChunk::Owned(owned_chunk);

        let serialized = data_chunk.serialize();
        let deserialized = OwnedDataChunk::deserialize(&serialized)?;

        assert_eq!(deserialized.data_ref(), original_data);

        Ok(())
    }
}
