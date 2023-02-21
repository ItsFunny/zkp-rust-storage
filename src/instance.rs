use std::collections::HashMap;
use std::path::Path;
pub use failure::Error;

extern crate merk;

use merk::*;
use merk::proofs::Query;
use merk::proofs::query::Map;
use crate::error::ErrorEnums;
use crate::middleware::Operation;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Order {}


pub trait DB {
    fn get(&self, k: Vec<u8>) -> Result<Option<Vec<u8>>>;
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> Result<()>;
    // fn batch_operation(&mut self, ops: Vec<Operation>) -> Result<()>;
    fn delete(&mut self, k: Vec<u8>) -> Result<()>;
}

pub trait TreeDB: DB {
    fn prove(&self, req: ProveRequest) -> Result<ProveResponse>;
    fn verify(&self, req: VerifyRequest) -> Result<VerifyResponse>;
    fn commit(&mut self, operations: Vec<Operation>) -> Result<()>;
    fn root_hash(&self) -> [u8; 32];
}


#[derive(Default)]
pub struct ProveRequest {
    pub query: Vec<Vec<u8>>,
}

impl ProveRequest {
    pub fn insert(&mut self, k: Vec<u8>) {
        self.query.push(k)
    }
}


pub struct ProveResponse {
    pub proof: Vec<u8>,
}

pub struct VerifyRequest {
    pub proof: Vec<u8>,
    pub expected_root: [u8; 32],
    pub kv: HashMap<Vec<u8>, Vec<u8>>,
}

impl VerifyRequest {
    pub fn new(proof: Vec<u8>, expected_root: [u8; 32]) -> Self {
        Self { proof, expected_root, kv: Default::default() }
    }

    pub fn insert(&mut self, k: Vec<u8>, v: Vec<u8>) {
        self.kv.insert(k, v);
    }
}


#[derive(Default)]
pub struct VerifyResponse {
    pub valid: bool,
}


pub struct MerkleTreeDB {
    m: Merk,
}

impl MerkleTreeDB {
    pub fn new(m: Merk) -> Self {
        Self { m }
    }
    pub fn new_with_path<P: AsRef<Path>>(p: P) -> Self {
        let merk = Merk::open(p).expect("fail to create ");
        Self { m: merk }
    }
}

impl DB for MerkleTreeDB {
    fn get(&self, k: Vec<u8>) -> Result<Option<Vec<u8>>> {
        self.m.get(k.as_slice()).map_err(|e| {
            ErrorEnums::Unknown.into()
        })
    }

    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> Result<()> {
        self.m.apply(&vec![(k, Op::Put(v))], &[]).map_err(|e| {
            ErrorEnums::Unknown.into()
        })
    }


    fn delete(&mut self, k: Vec<u8>) -> Result<()> {
        self.m.apply(&vec![(k, Op::Delete)], &[]).map_err(|e| {
            ErrorEnums::Unknown.into()
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


impl TreeDB for MerkleTreeDB {
    fn prove(&self, req: ProveRequest) -> Result<ProveResponse> {
        let mut q = Query::default();
        for k in req.query {
            q.insert_key(k.clone());
        }
        self.m.prove(q).map(|v| {
            ProveResponse { proof: v }
        }).map_err(|e| {
            ErrorEnums::Unknown.into()
        })
    }

    fn verify(&self, req: VerifyRequest) -> Result<VerifyResponse> {
        let res = merk::verify(req.proof.as_slice(), req.expected_root as Hash)?;
        // TODO,需要递归tree往上构造
        let mut ret = VerifyResponse::default();
        for (k, v) in req.kv {
            let value_opt = res.get(k.as_slice())?;
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

    fn commit(&mut self, mut operations: Vec<Operation>) -> Result<()> {
        self.batch_operation(operations)?;
        self.m.flush().map_err(|e| {
            ErrorEnums::Unknown.into()
        })
        // TODO,snapshot
    }

    fn root_hash(&self) -> [u8; 32] {
        self.m.root_hash() as [u8; 32]
    }
}

impl MerkleTreeDB {
    fn batch_operation(&mut self, ops: Vec<Operation>) -> Result<()> {
        let batches = to_batch(ops);
        self.m.apply(&batches, &[]).map_err(|e| {
            ErrorEnums::Unknown.into()
        })
    }
}

#[test]
pub fn test_merke() {
    // load or create a Merk store at the given path
    let mut merk = Merk::open("./merk.db").unwrap();

// apply some operations
    let batch = [
        (vec![9, 23, 3], Op::Put(vec![1, 2, 3])),
        (vec![4, 5, 6], Op::Delete)
    ];
    let res = merk.apply(&batch, &[]).expect("fail");

    let res = merk.get(&[1, 2, 3]).unwrap();
    println!("{:?}", String::from_utf8(res.unwrap()));
}