//! Error types for ZappyBase

use thiserror::Error;

/// Result type alias for ZappyBase operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for ZappyBase operations
#[derive(Debug, Error)]
pub enum Error {
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    #[error("Vector not found: {0}")]
    VectorNotFound(String),

    #[error("Duplicate vector ID: {0}")]
    DuplicateId(String),

    #[error("Index is empty, cannot search")]
    EmptyIndex,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Duplicate collection: {0}")]
    DuplicateCollection(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
