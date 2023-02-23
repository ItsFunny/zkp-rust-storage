use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::ops::IndexMut;
use std::sync::Arc;
use merk::{Batch, BatchEntry, Op};
use derive_builder::Builder;
use crate::error::ZKResult;
use crate::tree::couple::{ProveRequest, ProveResponse, VerifyRequest, VerifyResponse};
use crate::tree::operation::Operation;
use crate::tree::tree::{DB, TreeDB};


pub trait TreeMiddleware: TreeDB {
    type Inner: TreeMiddleware;
    fn inner(&self) -> &Self::Inner;
    fn clean(&mut self) -> ZKResult<()> {
        Ok(())
    }
}

pub enum DBType {
    Merkle
}


pub struct TransactionNode {
    version: u64,
    op: Operation,
}

impl TransactionNode {
    pub fn new(version: u64, op: Operation) -> Self {
        Self { version, op }
    }
}

pub struct Transaction(pub Vec<TransactionNode>);

impl Default for Transaction {
    fn default() -> Self {
        Self(Vec::new())
    }
}


////
pub struct DBMiddleware<D>
{
    db: D,
}


impl<D> DBMiddleware<D>
    where
        D: TreeDB,
{
    pub fn new(db: D) -> Self {
        Self { db }
    }
}

impl<D> TreeDB for DBMiddleware<D>
    where
        D: TreeDB,
{
    fn prove(&self, req: ProveRequest) -> ZKResult<ProveResponse> {
        self.db.prove(req)
    }

    fn verify(&self, req: VerifyRequest) -> ZKResult<VerifyResponse> {
        self.db.verify(req)
    }

    fn commit(&mut self, mut operations: Vec<Operation>) -> ZKResult<()> {
        self.db.commit(operations)
    }

    fn root_hash(&self) -> [u8; 32] {
        self.db.root_hash()
    }
}

impl<D> DB for DBMiddleware<D>
    where
        D: TreeDB,
{
    fn get(&self, k: Vec<u8>) -> ZKResult<Option<Vec<u8>>> {
        self.db.get(k.clone())
    }

    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> ZKResult<Vec<u8>> {
        self.db.set(k.clone(), v)?;
        Ok(k.clone())
    }


    fn delete(&mut self, k: Vec<u8>) -> ZKResult<()> {
        self.db.delete(k)
    }
}

impl<D> TreeMiddleware for DBMiddleware<D>
    where
        D: TreeDB,
{
    type Inner = Self;

    fn inner(&self) -> &Self::Inner {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use merk::{Hash, Merk};
    use crate::middleware::cache::CacheMiddleware;
    use crate::tree::tree::TreeDB;
    use crate::middleware::middleware::{DBMiddleware, TreeMiddleware};
    use crate::tree::tree::DB;
    use crate::tree::couple::{ProveRequest, VerifyRequest};
    use crate::tree::merkle::MerkleTreeDB;


    #[test]
    pub fn test_cache_get() {
        let mut cache = new_cache_merkle();
        let res = cache.get(vec![1, 2, 3]).expect("fail to get");
        assert_eq!(res, None)
    }

    #[test]
    pub fn test_set() {
        let mut cache = new_cache_merkle();
        cache.set(vec![1, 2, 3], vec![4, 5, 6]).expect("fail to set");
        let ret = cache.get(vec![1, 2, 3]).expect("fail to get").unwrap();
        assert_eq!(ret, vec![4, 5, 6])
    }

    #[test]
    pub fn test_commit() {
        let mut cache = new_cache_merkle();
        cache.set(vec![4, 5, 6], vec![1, 1, 1]).expect("fail to set ");
        cache.set(vec![1, 2, 3], vec![4, 5, 6]).expect("fail to set");

        cache.commit(vec![]).expect("fail to commit");
    }

    fn new_cache_merkle() -> impl TreeMiddleware {
        let mut merk = Merk::open("./merk.db").unwrap();
        let internal = MerkleTreeDB::new(merk);
        let db_middleware: DBMiddleware<MerkleTreeDB> = DBMiddleware::new(internal);
        let mut cache = CacheMiddleware::new(db_middleware);
        return cache;
    }

    #[test]
    pub fn test_prove_verify() {
        let mut mid = new_cache_merkle();
        mid.set(vec![1, 2, 3], vec![4, 5, 6]).expect("fail to set");
        mid.commit(vec![]).expect("fail to commit");
        let mut req = ProveRequest::default();
        req.insert(vec![1, 2, 3]);
        let root_hash = mid.root_hash();
        let prove = mid.prove(req).expect("fail to get prove");
        println!("{:?}", hex::encode(root_hash).to_string());
        let mut v_req = VerifyRequest::new(prove.proof, root_hash);
        v_req.insert(vec![1, 2, 3], vec![4, 5, 6]);
        let verify_resp = mid.verify(v_req).expect("fail to verify");
        assert_eq!(verify_resp.valid, true)
    }

    #[test]
    pub fn test_builder() {}
}

