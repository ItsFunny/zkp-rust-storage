use hash256_std_hasher::Hash256StdHasher;
use hash_db::{AsHashDB, HashDB, Hasher, Prefix};
use memory_db::prefixed_key;
use merk::{Merk, Op};

use sparse_merkle_tree::{
    blake2b::Blake2bHasher, default_store::DefaultStore,
    error::Error, MerkleProof,
    SparseMerkleTree, traits::Value, H256,
};
use tiny_keccak::{Hasher as KeccaHasher, Keccak};
use crate::error::{ZKError, ZKResult};
use crate::middleware::middleware::TreeMiddleware;
use crate::tree::couple::{ProveRequest, ProveResponse, VerifyRequest, VerifyResponse};
use crate::tree::operation::Operation;
use crate::tree::tree::{DB, TreeDB};

pub struct SMTMiddleware<M>
    where
        M: TreeMiddleware {
    inner: M,
}

// TODO, wrapped with db
pub struct SMTreeDB {
    m: Merk,
}

impl SMTreeDB {
    fn extend_prefix<H: Hasher>(&self, key: &H::Out, prefix: Prefix) -> Vec<u8> {
        prefixed_key::<H>(key, prefix)
    }
}

pub type KeccakHash = [u8; 32];

#[derive(Default, Debug, Clone, PartialEq)]
pub struct KeccakHasher;

impl Hasher for KeccakHasher {
    type Out = KeccakHash;

    type StdHasher = Hash256StdHasher;

    const LENGTH: usize = 32;

    fn hash(x: &[u8]) -> Self::Out {
        let mut keccak = Keccak::v256();
        keccak.update(x);
        let mut out = [0u8; 32];
        keccak.finalize(&mut out);
        out
    }
}


impl AsHashDB<KeccakHasher, Vec<u8>> for SMTreeDB {
    fn as_hash_db(&self) -> &dyn HashDB<KeccakHasher, Vec<u8>> {
        self
    }

    fn as_hash_db_mut<'a>(&'a mut self) -> &'a mut (dyn HashDB<KeccakHasher, Vec<u8>> + 'a) {
        self
    }
}

//
impl HashDB<KeccakHasher, Vec<u8>> for SMTreeDB {
    fn get(&self, key: &<KeccakHasher as Hasher>::Out, prefix: Prefix) -> Option<Vec<u8>> {
        let k = self.extend_prefix::<KeccakHasher>(key, prefix);
        self.m.get(k.as_slice()).map_or_else(|e| {
            None
        }, |v| {
            v
        })
    }

    fn contains(&self, key: &<KeccakHasher as Hasher>::Out, prefix: Prefix) -> bool {
        self.get(key, prefix).map_or_else(|| { false }, |v| {
            true
        })
    }

    fn insert(&mut self, _: Prefix, value: &[u8]) -> <KeccakHasher as Hasher>::Out {
        let key = KeccakHasher::hash(value);
        self.m.apply(&vec![(key.to_vec(), Op::Put(value.to_vec()))], &[]).expect("fail to set");
        key
    }

    fn emplace(&mut self, key: <KeccakHasher as Hasher>::Out, prefix: Prefix, value: Vec<u8>) {
        let key = self.extend_prefix::<KeccakHasher>(&key, prefix);
        self.m.apply(&vec![(key, Op::Put(value))], &[]).expect("fail to set");
    }

    fn remove(&mut self, key: &<KeccakHasher as Hasher>::Out, prefix: Prefix) {
        let key = self.extend_prefix::<KeccakHasher>(&key, prefix);
        self.m.apply(&vec![(key, Op::Delete)], &[]).expect("fail to delete")
    }
}


unsafe impl Send for SMTreeDB {}

unsafe impl Sync for SMTreeDB {}

impl<M> TreeDB for SMTMiddleware<M> where M: TreeMiddleware {
    fn prove(&self, req: ProveRequest) -> ZKResult<ProveResponse> {
        todo!()
    }

    fn verify(&self, req: VerifyRequest) -> ZKResult<VerifyResponse> {
        todo!()
    }

    fn commit(&mut self, operations: Vec<Operation>) -> ZKResult<()> {
        todo!()
    }

    fn root_hash(&self) -> [u8; 32] {
        todo!()
    }
}

impl<M> DB for SMTMiddleware<M> where M: TreeMiddleware {
    fn get(&self, k: Vec<u8>) -> ZKResult<Option<Vec<u8>>> {
        todo!()
    }

    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> ZKResult<Vec<u8>> {
        todo!()
    }

    fn delete(&mut self, k: Vec<u8>) -> ZKResult<()> {
        todo!()
    }
}

impl<M> TreeMiddleware for SMTMiddleware<M>
    where
        M: TreeMiddleware,
{
    type Inner = M;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn test_st() {}
}