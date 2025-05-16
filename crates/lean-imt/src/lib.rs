//! # Lean IMT
//!
//! LeanIMT provides an optimized incremental Merkle tree (IMT) implementation tailored for efficient, binary-based hashing without relying on zero values for incomplete nodes. It dynamically adjusts its depth based on leaf insertions, significantly reducing computational overhead.
//!
//! ## Quick Start
//!
//! Install the `zk-kit-lean-imt` crate with `cargo`:
//!
//! ```commandline
//! cargo add zk-kit-lean-imt
//! ```
//!
//! ## Example
//! ```rust
//! use lean_imt::hashed_tree::{HashedLeanIMT, LeanIMTHasher};
//! use std::collections::hash_map::DefaultHasher;
//! use std::hash::{Hash, Hasher};
//!
//! struct SampleHasher;
//!
//! impl LeanIMTHasher<32> for SampleHasher {
//!     fn hash(input: &[u8]) -> [u8; 32] {
//!         let mut hasher = DefaultHasher::new();
//!         input.hash(&mut hasher);
//!         let h = hasher.finish();
//!
//!         let mut result = [0u8; 32];
//!         result[..8].copy_from_slice(&h.to_le_bytes());
//!         result
//!     }
//! }
//!
//! let mut tree = HashedLeanIMT::<32, SampleHasher>::new(&[], SampleHasher).unwrap();
//!
//! tree.insert(&[1; 32]);
//! tree.insert(&[2; 32]);
//! tree.insert_many(&[[3; 32], [4; 32], [5; 32]]);
//!
//! println!("Tree root: {:?}", tree.root().unwrap());
//! println!("Tree depth: {}", tree.depth());
//!
//! let proof = tree.generate_proof(3).unwrap();
//! assert!(HashedLeanIMT::<32, SampleHasher>::verify_proof(&proof));
//! ```

pub mod hashed_tree;
pub mod lean_imt;

#[cfg(feature = "stateless")]
pub mod stateless;
