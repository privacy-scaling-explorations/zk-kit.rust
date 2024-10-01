pub mod database;
pub mod hasher;
pub mod tree;

use std::fmt::{Debug, Display};

pub use database::*;
pub use hasher::*;
pub use tree::MerkleTree;

/// Denotes keys in a database
pub type DBKey = [u8; 8];

/// Denotes values in a database
pub type Value = Vec<u8>;

/// Denotes pmtree Merkle tree errors
#[derive(Debug)]
pub enum TreeErrorKind {
    MerkleTreeIsFull,
    InvalidKey,
    IndexOutOfBounds,
    CustomError(String),
}

/// Denotes pmtree database errors
#[derive(Debug)]
pub enum DatabaseErrorKind {
    CannotLoadDatabase,
    DatabaseExists,
    CustomError(String),
}

/// Denotes pmtree errors
#[derive(Debug)]
pub enum PmtreeErrorKind {
    /// Error in database
    DatabaseError(DatabaseErrorKind),
    /// Error in tree
    TreeError(TreeErrorKind),
    /// Custom error
    CustomError(String),
}

impl Display for PmtreeErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PmtreeErrorKind::DatabaseError(e) => write!(f, "Database error: {e:?}"),
            PmtreeErrorKind::TreeError(e) => write!(f, "Tree error: {e:?}"),
            PmtreeErrorKind::CustomError(e) => write!(f, "Custom error: {e:?}"),
        }
    }
}

impl std::error::Error for PmtreeErrorKind {}

/// Custom `Result` type with custom `Error` type
pub type PmtreeResult<T> = std::result::Result<T, PmtreeErrorKind>;
