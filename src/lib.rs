pub mod aligned;
pub mod deserializer;
pub mod error;
pub mod serializer;
pub use aligned::AlignedDataChunk;
pub use error::PsDataChunkError;
pub use ps_cypher::Compressor;
pub use ps_mbuf::Mbuf;

#[inline(always)]
/// returns the first `50` bytes of a [str] as a `[u8; 50]`
pub fn convert_hash(hash: &str) -> Result<[u8; 50], PsDataChunkError> {
    PsDataChunkError::map_result(
        hash.as_bytes()[..50].try_into(),
        PsDataChunkError::HashConversionError,
    )
}

#[derive(rkyv::Archive, rkyv::Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// represents an owned chunk of data
pub struct OwnedDataChunk {
    pub hash: [u8; 50],
    pub data: Vec<u8>,
}

/// represents a chunk of data that is either owned or pointed to
pub enum DataChunk<'lt> {
    Mbuf(&'lt mut Mbuf<'lt, [u8; 50], u8>),
    Owned(OwnedDataChunk),
    Aligned(AlignedDataChunk),
}

/// represents an encrypted chunk of data and the key needed to decrypt it
pub struct EncryptedDataChunk {
    pub chunk: OwnedDataChunk,
    pub key: [u8; 50],
}

impl OwnedDataChunk {
    #[inline(always)]
    /// converts this `OwnedDataChunk` into a `Vec<u8>`
    /// - extends `self.hash`
    /// - returns `self.data`
    pub fn serialize_into(mut self) -> Vec<u8> {
        serializer::serialize_vec_with_known_hash(&mut self.data, &self.hash);

        return self.data;
    }

    #[inline(always)]
    /// serializes this `OwnedDataChunk` into a newly allocated `Vec<u8>`
    /// - allocated a new `Vec<u8>`
    /// - copies `self.data` into the new `Vec<u8>`
    /// - copies `self.hash` into the new `Vec<u8>`
    /// - returns the new `Vec<u8>`
    pub fn serialize(&self) -> Vec<u8> {
        serializer::serialize_bytes_with_known_hash(&self.data, &self.hash)
    }

    #[inline(always)]
    /// - converts a `Vec<u8>` into an `OwnedDataChunk`
    /// - performs hash validation
    pub fn deserialize_from(data: Vec<u8>) -> Result<Self, PsDataChunkError> {
        let (data, hash) = deserializer::deserialize_vec_to_parts(data)?;

        Ok(Self {
            data,
            hash: hash.into(),
        })
    }

    #[inline(always)]
    /// Copies `data` into a new `Vec<u8>` and deserializes it into an `OwnedDataChunk`.
    pub fn deserialize(data: &[u8]) -> Result<Self, PsDataChunkError> {
        Self::deserialize_from(data.to_vec())
    }

    #[inline(always)]
    /// Decrypts into an `OwnedDataChunk` with the given `key`
    /// - performs hash validation
    /// - fails if `key` not correct
    pub fn decrypt_bytes(
        encrypted: &[u8],
        key: &[u8],
        compressor: &mut Compressor,
    ) -> Result<Self, PsDataChunkError> {
        let decrypted = ps_cypher::decrypt(encrypted, key, compressor)?;

        Self::deserialize_from(decrypted)
    }

    #[inline(always)]
    /// Decrypts an `OwnedDataChunk` with the given `key`.
    /// - performs hash validation
    /// - fails if `key` not correct
    pub fn decrypt(
        &self,
        key: &[u8],
        compressor: &mut Compressor,
    ) -> Result<Self, PsDataChunkError> {
        Self::decrypt_bytes(&self.data, key, compressor)
    }

    #[inline(always)]
    /// Encrypts a serialized [DataChunk].
    pub fn encrypt_bytes(
        bytes: &[u8],
        compressor: &mut Compressor,
    ) -> Result<EncryptedDataChunk, PsDataChunkError> {
        let encrypted = ps_cypher::encrypt(bytes, compressor)?;

        Ok(EncryptedDataChunk {
            chunk: OwnedDataChunk {
                data: encrypted.bytes,
                hash: encrypted.hash.into(),
            },
            key: encrypted.key.into(),
        })
    }

    #[inline(always)]
    /// Encrypts this [DataChunk].
    pub fn encrypt(
        &self,
        compressor: &mut Compressor,
    ) -> Result<EncryptedDataChunk, PsDataChunkError> {
        Self::encrypt_bytes(&self.serialize(), compressor)
    }

    #[inline(always)]
    /// Encrypts this [DataChunk].
    /// - optimized by using `self.data` as the serialization buffer
    pub fn encrypt_mut(
        &mut self,
        compressor: &mut Compressor,
    ) -> Result<EncryptedDataChunk, PsDataChunkError> {
        let data_length = self.data.len();

        serializer::serialize_vec_with_known_hash(&mut self.data, &self.hash);

        let encrypted = Self::encrypt_bytes(&self.data, compressor);

        self.data.truncate(data_length);

        return encrypted;
    }
}

impl<'lt> Into<DataChunk<'lt>> for OwnedDataChunk {
    fn into(self) -> DataChunk<'lt> {
        DataChunk::Owned(self)
    }
}

impl<'lt> Into<OwnedDataChunk> for DataChunk<'lt> {
    fn into(self) -> OwnedDataChunk {
        match self {
            Self::Mbuf(_) => self.to_owned(),
            Self::Owned(owned) => owned,
            Self::Aligned(_) => self.to_owned(),
        }
    }
}

impl<'lt> DataChunk<'lt> {
    #[inline(always)]
    /// Unwraps this [DataChunk] into an `OwnedDataChunk`.
    /// - `DataChunk::Mbuf()` allocates a new `OwnedDataChunk`.
    pub fn into_owned(self) -> OwnedDataChunk {
        match self {
            DataChunk::Mbuf(mbuf) => OwnedDataChunk {
                data: mbuf.to_vec(),
                hash: mbuf.get_metadata().to_owned(),
            },
            DataChunk::Owned(chunk) => chunk,
            DataChunk::Aligned(aligned) => aligned.into(),
        }
    }

    #[inline(always)]
    /// Gets an owned copy of this [DataChunk].
    pub fn to_owned(&self) -> OwnedDataChunk {
        match self {
            DataChunk::Mbuf(mbuf) => OwnedDataChunk {
                data: mbuf.to_vec(),
                hash: mbuf.get_metadata().to_owned(),
            },
            DataChunk::Owned(chunk) => chunk.clone(),
            DataChunk::Aligned(aligned) => OwnedDataChunk {
                data: aligned.data().to_vec(),
                hash: aligned.hash(),
            },
        }
    }

    #[inline(always)]
    /// convert this [DataChunk] into a `Vec<u8>`
    /// - `DataChunk::Mbuf()` is copied via `.to_owned()`
    pub fn serialize_into(self) -> Vec<u8> {
        match self {
            Self::Mbuf(_) => self.to_owned().serialize_into(),
            Self::Owned(chunk) => chunk.serialize_into(),
            Self::Aligned(aligned) => aligned.serialize_into().to_vec(),
        }
    }

    #[inline(always)]
    /// Serializes this [DataChunk] into a new `Vec<u8>`.
    pub fn serialize(&self) -> Vec<u8> {
        self.to_owned().serialize_into()
    }

    #[inline(always)]
    /// Decrypts this [DataChunk] with a given key.
    pub fn decrypt(
        &self,
        key: &[u8],
        compressor: &mut Compressor,
    ) -> Result<Self, PsDataChunkError> {
        let owned = match self {
            Self::Mbuf(mbuf) => OwnedDataChunk::decrypt_bytes(&mbuf[..], key, compressor),
            Self::Owned(chunk) => chunk.decrypt(key, compressor),
            Self::Aligned(aligned) => {
                OwnedDataChunk::decrypt_bytes(aligned.data(), key, compressor)
            }
        }?;

        Ok(owned.into())
    }

    #[inline(always)]
    /// Encrypts this [DataChunk].
    pub fn encrypt(
        &self,
        compressor: &mut Compressor,
    ) -> Result<EncryptedDataChunk, PsDataChunkError> {
        OwnedDataChunk::encrypt_bytes(&self.serialize(), compressor)
    }

    #[inline(always)]
    /// Encrypts this [DataChunk] using `self.data` if owned.
    pub fn encrypt_mut(
        &mut self,
        compressor: &mut Compressor,
    ) -> Result<EncryptedDataChunk, PsDataChunkError> {
        match self {
            DataChunk::Mbuf(_) => self.encrypt(compressor),
            DataChunk::Owned(chunk) => chunk.encrypt_mut(compressor),
            DataChunk::Aligned(_) => self.encrypt(compressor),
        }
    }
}

impl EncryptedDataChunk {
    /// Decrypts this `EncryptedDataChunk`.
    pub fn decrypt(&self, compressor: &mut Compressor) -> Result<OwnedDataChunk, PsDataChunkError> {
        OwnedDataChunk::decrypt(&self.chunk, &self.key, compressor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() -> Result<(), PsDataChunkError> {
        let mut compressor = Compressor::new();
        let original_data = "Neboť tak Bůh miluje svět, že dal [svého] jediného Syna, aby žádný, kdo v něho věří, nezahynul, ale měl život věčný. Vždyť Bůh neposlal [svého] Syna na svět, aby svět odsoudil, ale aby byl svět skrze něj zachráněn.".as_bytes().to_owned();

        let data_chunk = DataChunk::Owned(OwnedDataChunk {
            hash: ps_hash::hash(&original_data).into(),
            data: original_data.clone(),
        });

        let encrypted_chunk = data_chunk.encrypt(&mut compressor)?;
        let decrypted_chunk = encrypted_chunk.decrypt(&mut compressor)?;

        assert_eq!(decrypted_chunk.data, original_data);

        Ok(())
    }

    #[test]
    fn test_serialization() -> Result<(), PsDataChunkError> {
        let original_data = vec![1, 2, 3, 4, 5];
        let hash = ps_hash::hash(&original_data).into();
        let owned_chunk = OwnedDataChunk {
            hash,
            data: original_data.clone(),
        };
        let data_chunk = DataChunk::Owned(owned_chunk);

        let serialized = data_chunk.serialize();
        let deserialized = OwnedDataChunk::deserialize(&serialized)?;

        assert_eq!(deserialized.data, original_data);

        Ok(())
    }
}
