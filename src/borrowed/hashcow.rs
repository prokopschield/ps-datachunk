use ps_hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum HashCow<'lt> {
    Borrowed(&'lt Hash),
    Owned(Arc<Hash>),
}

impl<'lt, T: Into<Hash>> From<T> for HashCow<'lt> {
    fn from(value: T) -> Self {
        Self::Owned(Arc::from(value.into()))
    }
}

impl<'lt> HashCow<'lt> {
    pub fn from_arc(hash: Arc<Hash>) -> Self {
        Self::Owned(hash)
    }

    pub fn from_ref(hash: &'lt Hash) -> Self {
        Self::Borrowed(hash)
    }
}

impl<'lt> Into<Arc<Hash>> for &HashCow<'lt> {
    fn into(self) -> Arc<Hash> {
        match self {
            HashCow::Borrowed(borrowed) => Arc::from(**borrowed),
            HashCow::Owned(owned) => owned.clone(),
        }
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
