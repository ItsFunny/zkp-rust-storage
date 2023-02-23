use std::path::Path;
use merk::{BatchEntry, Hash, Merk, Op};
use merk::proofs::Query;
use crate::error::{ErrorEnums, ErrorEnumsStruct, ZKError, ZKResult};
use crate::tree::couple::{ProveRequest, ProveResponse, VerifyRequest, VerifyResponse};
use crate::tree::operation::Operation;
use crate::tree::tree::{DB, TreeDB};

pub struct MerkleRocksDB {
    m: Merk,
}

unsafe impl Send for MerkleRocksDB {}

unsafe impl Sync for MerkleRocksDB {}

impl MerkleRocksDB {
    pub fn new(m: Merk) -> Self {
        Self { m }
    }
    pub fn new_with_path<P: AsRef<Path>>(p: P) -> Self {
        let merk = Merk::open(p).expect("fail to create ");
        Self { m: merk }
    }
}

impl DB for MerkleRocksDB {
    fn get(&self, k: Vec<u8>) -> ZKResult<Option<Vec<u8>>> {
        self.m.get(k.as_slice()).map_err(|e| {
            ZKError::from(ErrorEnumsStruct::UNKNOWN).with_error(Box::new(e))
        })
    }

    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> ZKResult<Vec<u8>> {
        self.m.apply(&vec![(k.clone(), Op::Put(v))], &[]).map_err(|e| {
            ZKError::from(ErrorEnumsStruct::UNKNOWN).with_error(Box::new(e))
        }).map(|_| {
            k.clone()
        })
    }


    fn delete(&mut self, k: Vec<u8>) -> ZKResult<()> {
        self.m.apply(&vec![(k, Op::Delete)], &[]).map_err(|e| {
            ZKError::from(ErrorEnumsStruct::UNKNOWN).with_error(Box::new(e))
        })
    }
}

fn to_batch(ops: Vec<Operation>) -> Vec<BatchEntry> {
    let mut batch = Vec::new();
    for val in ops {
        match val {
            Operation::Set(k, v) => batch.push((k, Op::Put(v))),
            Operation::Delete(k) => batch.push((k, Op::Delete)),
        }
    }
    batch
}


impl TreeDB for MerkleRocksDB {
    fn prove(&self, req: ProveRequest) -> ZKResult<ProveResponse> {
        let mut q = Query::default();
        for k in req.query {
            q.insert_key(k.clone());
        }
        self.m.prove(q).map(|v| {
            ProveResponse { proof: v }
        }).map_err(|e| {
            ZKError::from(ErrorEnumsStruct::UNKNOWN).with_error(Box::new(e))
        })
    }

    fn verify(&self, req: VerifyRequest) -> ZKResult<VerifyResponse> {
        let res = merk::verify(req.proof.as_slice(), req.expected_root as Hash).map_err(|e| {
            ZKError::from(ErrorEnumsStruct::UNKNOWN).with_error(Box::new(e))
        })?;
        // TODO,需要递归tree往上构造
        let mut ret = VerifyResponse::default();
        for (k, v) in req.kv {
            let value_opt = res.get(k.as_slice()).map_err(|e| {
                ZKError::from(ErrorEnumsStruct::UNKNOWN).with_error(Box::new(e))
            })?;
            if let Some(value) = value_opt {
                if value != v {
                    ret.valid = false;
                    return Ok(ret);
                }
            }
        }
        ret.valid = true;
        Ok(ret)
    }

    fn commit(&mut self, mut operations: Vec<Operation>) -> ZKResult<()> {
        self.batch_operation(operations)?;
        self.m.flush().map_err(|e| {
            ZKError::from(ErrorEnumsStruct::UNKNOWN).with_error(Box::new(e))
        })
        // TODO,snapshot
    }

    fn root_hash(&self) -> [u8; 32] {
        self.m.root_hash() as [u8; 32]
    }
}

impl MerkleRocksDB {
    fn batch_operation(&mut self, ops: Vec<Operation>) -> ZKResult<()> {
        let batches = to_batch(ops);
        self.m.apply(&batches, &[]).map_err(|e| {
            ZKError::from(ErrorEnumsStruct::UNKNOWN).with_error(Box::new(e))
        })
    }
}

#[test]
pub fn test_merke() {
    // load or create a Merk store at the given path
    let mut merk = Merk::open("./merk.db").unwrap();

// apply some operations
    let batch = [
        (vec![1, 2, 3], Op::Put(vec![1, 2, 3])),
        (vec![4, 5, 6], Op::Delete)
    ];
    let res = merk.apply(&batch, &[]).expect("fail");

    let res = merk.get(&[1, 2, 3]).unwrap();
    println!("{:?}", String::from_utf8(res.unwrap()));
}