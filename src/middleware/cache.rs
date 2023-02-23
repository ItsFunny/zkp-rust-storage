use std::collections::BTreeMap;
use crate::error::ZKResult;
use crate::middleware::middleware::TreeMiddleware;
use crate::tree::couple::{ProveRequest, ProveResponse, VerifyRequest, VerifyResponse};
use crate::tree::operation::Operation;
use crate::tree::tree::{DB, TreeDB};

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

// #[derive(Default)]
// pub struct cacheInner {
//     transactions: Transaction,
// }
//
// impl cacheInner {
//     pub fn find(self, k: Vec<u8>) -> Option<Vec<u8>> {
//         let mut ret: Vec<u8> = vec![];
//         let mut find = false;
//         for v in self.transactions.0 {
//             match v.op {
//                 Operation::Set(key, value) => {
//                     if k == key {
//                         find = true;
//                         ret = value;
//                     }
//                 }
//                 _ => {}
//             }
//         }
//         if find {
//             return Some(ret);
//         }
//         None
//     }
// }


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
    fn get(&self, k: Vec<u8>) -> ZKResult<Option<Vec<u8>>> {
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

    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> ZKResult<Vec<u8>> {
        self.map.as_mut().unwrap().insert(k.clone(), Some(v));
        Ok(k.clone())
    }


    fn delete(&mut self, k: Vec<u8>) -> ZKResult<()> {
        self.map.as_mut().unwrap().insert(k, None);
        Ok(())
    }
}

impl<M> TreeDB for CacheMiddleware<M>
    where
        M: TreeMiddleware, {
    fn prove(&self, req: ProveRequest) -> ZKResult<ProveResponse> {
        self.inner.prove(req)
    }

    fn verify(&self, req: VerifyRequest) -> ZKResult<VerifyResponse> {
        self.inner.verify(req)
    }

    fn commit(&mut self, mut operations: Vec<Operation>) -> ZKResult<()> {
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

    fn clean(&mut self) -> ZKResult<()> {
        self.map = Some(Map::new());
        Ok(())
    }
}