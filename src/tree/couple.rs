use std::collections::HashMap;

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
