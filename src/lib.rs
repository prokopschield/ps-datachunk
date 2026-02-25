#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
pub mod aligned;
pub mod borrowed;
pub mod cow;
pub mod encrypted;
pub mod error;
pub mod mbuf;
pub mod owned;
pub mod serialized;
pub mod typed;
pub mod utils;
pub use aligned::AlignedDataChunk;
pub use borrowed::BorrowedDataChunk;
pub use bytes::Bytes;
pub use cow::CowDataChunk;
pub use encrypted::EncryptedDataChunk;
pub use error::PsDataChunkError;
pub use error::Result;
pub use mbuf::MbufDataChunk;
pub use owned::OwnedDataChunk;
pub use ps_hash::Hash;
pub use ps_mbuf::Mbuf;
pub use serialized::SerializedDataChunk;
pub use typed::ToDataChunk;
pub use typed::ToTypedDataChunk;
pub use typed::TypedDataChunk;

use std::sync::Arc;

/// Represents any representation of a chunk of data.
///
/// # Invariants
///
/// Implementers must treat the bytes and hash exposed through `&self` as immutable and stable.
/// In practice this means:
/// - Repeated calls to [`Self::data_ref`] must point to the same logical bytes while `self` is borrowed.
/// - Repeated calls to [`Self::hash_ref`] must return the hash for those same bytes.
/// - Neither value may change through interior mutability while `self` is borrowed.
///
/// `TypedDataChunk` relies on this contract for its unchecked deref fast path.
pub trait DataChunk
where
    Self: Sized,
{
    /// Returns a stable view of the underlying bytes.
    fn data_ref(&self) -> &[u8];
    /// Returns a stable view of the hash corresponding to [`Self::data_ref`].
    fn hash_ref(&self) -> &Hash;

    fn hash(&self) -> Hash {
        *self.hash_ref()
    }

    fn encrypt(&self) -> Result<EncryptedDataChunk> {
        self.serialize()?.encrypt()
    }

    fn decrypt(&self, key: &Hash) -> Result<SerializedDataChunk> {
        utils::decrypt(self.data_ref(), key)
    }

    fn borrow(&self) -> BorrowedDataChunk<'_> {
        BorrowedDataChunk::from_parts_unchecked(self.data_ref(), self.hash())
    }

    fn serialize(&self) -> Result<SerializedDataChunk> {
        SerializedDataChunk::from_parts_unchecked(self.data_ref(), self.hash())
    }

    /// Transforms this [`DataChunk`] into [`Bytes`].
    fn into_bytes(self) -> Bytes {
        Bytes::from_owner(Arc::from(self.data_ref()))
    }

    /// Copies this [`DataChunk`] into a new [`OwnedDataChunk`].
    fn into_owned(self) -> OwnedDataChunk {
        OwnedDataChunk::from_data_and_hash_unchecked(Arc::from(self.data_ref()), self.hash())
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

    // Keep crate-root tests for cross-module flow behavior.
    #[test]
    fn test_encryption_decryption() -> Result<()> {
        let original_data = "Neboť tak Bůh miluje svět, že dal [svého] jediného Syna, aby žádný, kdo v něho věří, nezahynul, ale měl život věčný. Vždyť Bůh neposlal [svého] Syna na svět, aby svět odsoudil, ale aby byl svět skrze něj zachráněn.".as_bytes().to_owned();

        let data_chunk = BorrowedDataChunk::from_data(&original_data)?;

        let encrypted_chunk = data_chunk.encrypt()?;
        let decrypted_chunk = encrypted_chunk.decrypt()?;

        assert_eq!(decrypted_chunk.data_ref(), original_data);

        Ok(())
    }
}
