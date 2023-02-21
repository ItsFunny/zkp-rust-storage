use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;
use merk::{Batch, BatchEntry, Op};
use crate::instance::{DB, ProveRequest, ProveResponse, TreeDB, VerifyRequest, VerifyResponse};

pub trait TreeMiddleware: TreeDB {
    type Inner: TreeMiddleware;
    fn inner(&self) -> &Self::Inner;
    fn clean(&mut self) -> crate::instance::Result<()> {
        Ok(())
    }
}

pub enum DBType {
    Merkle
}

pub struct BuildOption {
    db_type: Option<DBType>,
    db_path: Option<String>,
}


pub enum Operation {
    Set(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
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

type Map = BTreeMap<Vec<u8>, Option<Vec<u8>>>;

pub struct CacheMiddleware<M>
    where
        M: TreeMiddleware,
{
    // c: Cell<Option<cacheInner>>,
    map: Option<Map>,
    inner: M,
    version: u64,
}

#[derive(Default)]
pub struct cacheInner {
    transactions: Transaction,
}

impl cacheInner {
    pub fn find(self, k: Vec<u8>) -> Option<Vec<u8>> {
        let mut ret: Vec<u8> = vec![];
        let mut find = false;
        for v in self.transactions.0 {
            match v.op {
                Operation::Set(key, value) => {
                    if k == key {
                        find = true;
                        ret = value;
                    }
                }
                _ => {}
            }
        }
        if find {
            return Some(ret);
        }
        None
    }
}


impl<M> CacheMiddleware<M>
    where
        M: TreeMiddleware,
{
    pub fn new(mid: M) -> CacheMiddleware<M> {
        Self {
            map: Some(Default::default()),
            inner: mid,
            version: 0,
        }
    }
}

impl<M> DB for CacheMiddleware<M>
    where
        M: TreeMiddleware,
{
    fn get(&self, k: Vec<u8>) -> crate::instance::Result<Option<Vec<u8>>> {
        match self.map.as_ref().unwrap().get(k.as_slice()) {
            Some(Some(value)) => Ok(Some(value.clone())),
            Some(None) => Ok(None),
            None => {
                self.inner.get(k.clone()).map(|v| {
                    if let Some(value) = v {
                        Some(value)
                    } else {
                        None
                    }
                })
            }
        }
    }

    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> crate::instance::Result<()> {
        self.map.as_mut().unwrap().insert(k, Some(v));
        Ok(())
    }


    fn delete(&mut self, k: Vec<u8>) -> crate::instance::Result<()> {
        self.map.as_mut().unwrap().insert(k, None);
        Ok(())
    }
}

impl<M> TreeDB for CacheMiddleware<M>
    where
        M: TreeMiddleware, {
    fn prove(&self, req: ProveRequest) -> crate::instance::Result<ProveResponse> {
        self.inner.prove(req)
    }

    fn verify(&self, req: VerifyRequest) -> crate::instance::Result<VerifyResponse> {
        self.inner.verify(req)
    }

    fn commit(&mut self, mut operations: Vec<Operation>) -> crate::instance::Result<()> {
        let map = self.map.take().unwrap();
        self.map = Some(Map::new());

        for (k, v) in map {
            match v.clone() {
                Some(value) => operations.push(Operation::Set(k, value)),
                None => operations.push(Operation::Delete(k)),
            }
        }
        self.inner.commit(operations)
    }

    fn root_hash(&self) -> [u8; 32] {
        self.inner.root_hash()
    }
}


impl<M> TreeMiddleware for CacheMiddleware<M>
    where
        M: TreeMiddleware,
{
    type Inner = M;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    fn clean(&mut self) -> crate::instance::Result<()> {
        self.map = Some(Map::new());
        Ok(())
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
    fn prove(&self, req: ProveRequest) -> crate::instance::Result<ProveResponse> {
        self.db.prove(req)
    }

    fn verify(&self, req: VerifyRequest) -> crate::instance::Result<VerifyResponse> {
        self.db.verify(req)
    }

    fn commit(&mut self, mut operations: Vec<Operation>) -> crate::instance::Result<()> {
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
    fn get(&self, k: Vec<u8>) -> crate::instance::Result<Option<Vec<u8>>> {
        self.db.get(k)
    }

    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> crate::instance::Result<()> {
        self.db.set(k, v)
    }


    fn delete(&mut self, k: Vec<u8>) -> crate::instance::Result<()> {
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
    use crate::instance::{DB, MerkleTreeDB, ProveRequest, TreeDB, VerifyRequest};
    use crate::middleware::{CacheMiddleware, DBMiddleware, TreeMiddleware};

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
}

