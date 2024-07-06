use ps_hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum HashCow<'lt> {
    Borrowed(&'lt Hash),
    Owned(Arc<Hash>),
}

impl<'lt> HashCow<'lt> {
    pub fn from_arc(hash: Arc<Hash>) -> Self {
        Self::Owned(hash)
    }

    pub fn from_hash(hash: Hash) -> Self {
        Self::from_arc(hash.into())
    }

    pub fn from_ref(hash: &'lt Hash) -> Self {
        Self::Borrowed(hash)
    }

    pub fn to_arc(&self) -> Arc<Hash> {
        self.into()
    }

    pub fn to_ref(&self) -> &Hash {
        self
    }
}

impl<'lt> From<Arc<Hash>> for HashCow<'lt> {
    fn from(hash: Arc<Hash>) -> Self {
        Self::from_arc(hash)
    }
}

impl<'lt> From<Hash> for HashCow<'lt> {
    fn from(hash: Hash) -> Self {
        Self::from_hash(hash)
    }
}

impl<'lt> From<&'lt Hash> for HashCow<'lt> {
    fn from(hash: &'lt Hash) -> Self {
        Self::from_ref(hash)
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

impl<'lt> Into<Arc<Hash>> for HashCow<'lt> {
    fn into(self) -> Arc<Hash> {
        (&self).into()
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
