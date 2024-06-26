use ps_hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

pub enum HashCow<'lt> {
    Borrowed(&'lt Hash),
    Owned(Arc<Hash>),
}

impl<'lt, T: Into<Hash>> From<T> for HashCow<'lt> {
    fn from(value: T) -> Self {
        Self::Owned(Arc::from(value.into()))
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
