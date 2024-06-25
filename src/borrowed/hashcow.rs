use ps_hash::Hash;
use std::ops::Deref;

pub enum HashCow<'lt> {
    Borrowed(&'lt Hash),
    Owned(Box<Hash>),
}

impl<'lt> From<&'lt Hash> for HashCow<'lt> {
    fn from(hash: &'lt Hash) -> Self {
        Self::Borrowed(hash)
    }
}

impl<'lt> From<Box<Hash>> for HashCow<'lt> {
    fn from(hash: Box<Hash>) -> Self {
        Self::Owned(hash)
    }
}

impl<'lt> From<Hash> for HashCow<'lt> {
    fn from(hash: Hash) -> Self {
        Self::from(Box::from(hash))
    }
}

impl<'lt> Deref for HashCow<'lt> {
    type Target = Hash;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Borrowed(hash) => hash,
            Self::Owned(hash) => &hash,
        }
    }
}
