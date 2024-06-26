pub mod hashcow;
use crate::DataChunkTrait;
pub use hashcow::HashCow;

#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct BorrowedDataChunk<'lt> {
    data: &'lt [u8],
    hash: HashCow<'lt>,
}

impl<'lt> BorrowedDataChunk<'lt> {
    pub fn from_parts(data: &'lt [u8], hash: HashCow<'lt>) -> Self {
        Self { data, hash }
    }

    pub fn from_data(data: &'lt [u8]) -> Self {
        let hash = ps_hash::hash(data);

        Self::from_parts(data, hash.into())
    }
}

impl<'lt> DataChunkTrait for BorrowedDataChunk<'lt> {
    fn data_ref(&self) -> &[u8] {
        &self.data
    }
    fn hash_ref(&self) -> &[u8] {
        self.hash.as_bytes()
    }
    fn hash(&self) -> HashCow {
        (&self.hash).clone()
    }
}

impl<'lt, T: DataChunkTrait> From<&'lt T> for BorrowedDataChunk<'lt> {
    fn from(chunk: &'lt T) -> Self {
        Self::from_parts(chunk.data_ref(), chunk.hash())
    }
}
