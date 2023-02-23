use std::collections::HashMap;
use std::path::Path;

use hash256_std_hasher::Hash256StdHasher;
use hash_db::{AsHashDB, HashDB, Hasher, Prefix};
use memory_db::prefixed_key;
use tiny_keccak::{Hasher as _, Keccak};

extern crate merk;

use merk::*;
use merk::proofs::Query;
use merk::proofs::query::Map;
use crate::error::{ErrorEnums, ZKResult};
use crate::tree::couple::{ProveRequest, ProveResponse, VerifyRequest, VerifyResponse};
use crate::tree::operation::Operation;


pub trait DB {
    fn get(&self, k: Vec<u8>) -> ZKResult<Option<Vec<u8>>>;
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> ZKResult<Vec<u8>>;
    // fn batch_operation(&mut self, ops: Vec<Operation>) -> Result<()>;
    fn delete(&mut self, k: Vec<u8>) -> ZKResult<()>;
}

pub trait TreeDB: DB {
    fn prove(&self, req: ProveRequest) -> ZKResult<ProveResponse>;
    fn verify(&self, req: VerifyRequest) -> ZKResult<VerifyResponse>;
    fn commit(&mut self, operations: Vec<Operation>) -> ZKResult<()>;
    fn root_hash(&self) -> [u8; 32];
}


