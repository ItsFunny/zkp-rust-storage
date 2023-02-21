use crate::instance::{DB, ProveRequest, ProveResponse, TreeDB, VerifyRequest, VerifyResponse};
use crate::middleware::Operation;

pub struct TrieDB {}

impl DB for TrieDB {
    fn get(&self, k: Vec<u8>) -> crate::instance::Result<Option<Vec<u8>>> {
        todo!()
    }

    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> crate::instance::Result<()> {
        todo!()
    }

    fn delete(&mut self, k: Vec<u8>) -> crate::instance::Result<()> {
        todo!()
    }
}

impl TreeDB for TrieDB {
    fn prove(&self, req: ProveRequest) -> crate::instance::Result<ProveResponse> {
        todo!()
    }

    fn verify(&self, req: VerifyRequest) -> crate::instance::Result<VerifyResponse> {
        todo!()
    }

    fn commit(&mut self, operations: Vec<Operation>) -> crate::instance::Result<()> {
        todo!()
    }

    fn root_hash(&self) -> [u8; 32] {
        todo!()
    }
}