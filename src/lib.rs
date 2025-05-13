#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
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
pub trait DataChunk
where
    Self: Sized,
{
    fn data_ref(&self) -> &[u8];
    fn hash_ref(&self) -> &Hash;

    fn hash(&self) -> Arc<Hash>;

    fn encrypt(&self) -> Result<EncryptedDataChunk> {
        self.serialize()?.encrypt()
    }

    fn decrypt(&self, key: &[u8]) -> Result<SerializedDataChunk> {
        utils::decrypt(self.data_ref(), key)
    }

    fn borrow(&self) -> BorrowedDataChunk {
        BorrowedDataChunk::from_parts(self.data_ref(), self.hash())
    }

    fn serialize(&self) -> Result<SerializedDataChunk> {
        SerializedDataChunk::from_parts(self.data_ref(), self.hash())
    }

    /// Copies this [`DataChunk`] into a new [`OwnedDataChunk`].
    fn into_owned(self) -> OwnedDataChunk {
        OwnedDataChunk::from_data_and_hash(Arc::from(self.data_ref()), self.hash())
    }

    fn try_as<T: rkyv::Archive>(self) -> Result<TypedDataChunk<Self, T>>
    where
        T::Archived:
            for<'a> rkyv::bytecheck::CheckBytes<rkyv::api::high::HighValidator<'a, rancor::Error>>,
    {
        TypedDataChunk::<Self, T>::from_data_chunk(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() -> Result<()> {
        let original_data = "Neboť tak Bůh miluje svět, že dal [svého] jediného Syna, aby žádný, kdo v něho věří, nezahynul, ale měl život věčný. Vždyť Bůh neposlal [svého] Syna na svět, aby svět odsoudil, ale aby byl svět skrze něj zachráněn.".as_bytes().to_owned();

        let data_chunk = BorrowedDataChunk::from_data(&original_data)?;

        let encrypted_chunk = data_chunk.encrypt()?;
        let decrypted_chunk = encrypted_chunk.decrypt()?;

        assert_eq!(decrypted_chunk.data_ref(), original_data);

        Ok(())
    }

    #[test]
    fn test_serialization() -> Result<()> {
        let original_data = vec![1, 2, 3, 4, 5];
        let hash = ps_hash::hash(&original_data)?.into();
        let data_chunk = OwnedDataChunk::from_data_and_hash(original_data.clone(), hash);

        let serialized = data_chunk.serialize()?;

        assert_eq!(serialized.data_ref(), original_data);

        Ok(())
    }
}
