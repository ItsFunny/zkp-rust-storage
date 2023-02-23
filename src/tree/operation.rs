pub enum Operation {
    Set(Vec<u8>, Vec<u8>),
    Delete(Vec<u8>),
}
