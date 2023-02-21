pub use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorEnums {
    #[error("Unknown Error")]
    Unknown,
}