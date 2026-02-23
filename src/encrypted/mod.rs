use crate::utils;
use crate::DataChunk;
use crate::Result;
use crate::SerializedDataChunk;
use bytes::Bytes;
use ps_buffer::Buffer;
use ps_buffer::SharedBuffer;
use ps_cypher::Encrypted;
use ps_hash::Hash;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// represents an encrypted chunk of data and the key needed to decrypt it
pub struct EncryptedDataChunk {
    data: Buffer,
    hash: Hash,
    key: Hash,
}

impl EncryptedDataChunk {
    /// Decrypts this `EncryptedDataChunk`.
    pub fn decrypt(&self) -> Result<SerializedDataChunk> {
        utils::decrypt(self.data_ref(), &self.key)
    }

    #[must_use]
    pub const fn key(&self) -> Hash {
        self.key
    }

    #[must_use]
    pub const fn key_ref(&self) -> &Hash {
        &self.key
    }
}

impl DataChunk for EncryptedDataChunk {
    fn data_ref(&self) -> &[u8] {
        &self.data
    }
    fn hash_ref(&self) -> &Hash {
        &self.hash
    }
    fn hash(&self) -> Hash {
        self.hash
    }

    /// Transforms this [`DataChunk`] into [`Bytes`].
    fn into_bytes(self) -> Bytes {
        Bytes::from_owner(SharedBuffer::from(self.data))
    }

    /// Transforms this chunk into an [`OwnedDataChunk`]
    fn into_owned(self) -> crate::OwnedDataChunk {
        let Self { data, hash, key: _ } = self;

        crate::OwnedDataChunk::from_data_and_hash(data, hash)
    }
}

impl From<Encrypted> for EncryptedDataChunk {
    fn from(value: Encrypted) -> Self {
        Self {
            data: value.bytes,
            hash: value.hash,
            key: value.key,
        }
    }
}
