//! # Hashed LeanIMT
//!
//! Lean Incremental Merkle Tree hashing function wrapper.

use crate::lean_imt::{LeanIMT, LeanIMTError, MerkleProof};
use serde::{Deserialize, Serialize};

/// LeanIMT hasher trait.
pub trait LeanIMTHasher {
    fn hash(input: &[u8]) -> Vec<u8>;
}

/// Hashed LeanIMT struct.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HashedLeanIMT<H> {
    /// LeanIMT
    tree: LeanIMT,
    /// Hasher
    hasher: H,
}

impl<H> HashedLeanIMT<H>
where
    H: LeanIMTHasher,
{
    /// Creates a new tree with optional initial leaves.
    pub fn new(leaves: &[Vec<u8>], hasher: H) -> Result<Self, LeanIMTError> {
        let imt = LeanIMT::new(leaves, H::hash)?;

        Ok(Self { tree: imt, hasher })
    }

    /// Returns the root, if it exists.
    pub fn root(&self) -> Option<Vec<u8>> {
        self.tree.root()
    }

    /// Returns the tree depth.
    pub fn depth(&self) -> usize {
        self.tree.depth()
    }

    /// Returns the leaves.
    pub fn leaves(&self) -> &[Vec<u8>] {
        self.tree.leaves()
    }

    /// Returns the number of leaves.
    pub fn size(&self) -> usize {
        self.tree.size()
    }

    /// Returns the index of a leaf, if it exists.
    pub fn index_of(&self, leaf: &[u8]) -> Option<usize> {
        self.tree.index_of(leaf)
    }

    /// Checks if a leaf exists.
    pub fn contains(&self, leaf: &[u8]) -> bool {
        self.tree.contains(leaf)
    }

    /// Returns the leaf at the given index.
    pub fn get_leaf(&self, index: usize) -> Option<Vec<u8>> {
        self.tree.get_leaf(index)
    }

    /// Inserts a single leaf.
    pub fn insert(&mut self, leaf: &[u8]) {
        self.tree.insert(leaf, H::hash)
    }

    /// Inserts multiple leaves.
    pub fn insert_many(&mut self, leaves: &[Vec<u8>]) -> Result<(), LeanIMTError> {
        self.tree.insert_many(leaves, H::hash)
    }

    /// Updates a leaf at the given index.
    pub fn update(&mut self, index: usize, new_leaf: &[u8]) -> Result<(), LeanIMTError> {
        self.tree.update(index, new_leaf, H::hash)
    }

    /// Generates a Merkle proof for a leaf at the given index.
    pub fn generate_proof(&self, index: usize) -> Result<MerkleProof, LeanIMTError> {
        self.tree.generate_proof(index)
    }

    /// Verifies a Merkle proof.
    pub fn verify_proof(proof: &MerkleProof) -> bool {
        LeanIMT::verify_proof(proof, H::hash)
    }
}

#[cfg(test)]
mod tests {
    use super::{HashedLeanIMT, LeanIMTHasher};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    struct SampleHasher;

    impl LeanIMTHasher for SampleHasher {
        fn hash(input: &[u8]) -> Vec<u8> {
            let mut hasher = DefaultHasher::new();
            for byte in input {
                byte.hash(&mut hasher);
            }
            let hash = hasher.finish();

            let mut result = vec![0u8; 32];
            result[..8].copy_from_slice(&hash.to_le_bytes());
            result
        }
    }

    #[test]
    fn test_new_empty_tree() {
        let tree = HashedLeanIMT::new(&[], SampleHasher).unwrap();
        assert_eq!(tree.size(), 0);
        assert_eq!(tree.root(), None);
    }

    #[test]
    fn test_insert_leaves() {
        let mut tree = HashedLeanIMT::new(&[], SampleHasher).unwrap();

        // Insert a single leaf
        tree.insert(&[1; 32]);
        assert_eq!(tree.size(), 1);
        assert!(tree.contains(&[1; 32]));

        // Insert multiple leaves
        tree.insert_many(&[b"leaf2".to_vec(), b"leaf3".to_vec()])
            .unwrap();
        assert_eq!(tree.size(), 3);
        assert!(tree.contains(b"leaf2"));
        assert!(tree.contains(b"leaf3"));
    }

    #[test]
    fn test_update_leaf() {
        let mut tree =
            HashedLeanIMT::new(&[b"leaf1".to_vec(), b"leaf2".to_vec()], SampleHasher).unwrap();

        tree.update(0, b"updated_leaf").unwrap();
        assert!(!tree.contains(b"leaf1"));
        assert!(tree.contains(b"updated_leaf"));

        assert!(tree.contains(b"leaf2"));
    }

    #[test]
    fn test_merkle_proof() {
        let leaves = vec![
            b"leaf1".to_vec(),
            b"leaf2".to_vec(),
            b"leaf3".to_vec(),
            b"leaf4".to_vec(),
        ];

        let tree = HashedLeanIMT::new(&leaves, SampleHasher).unwrap();

        let proof = tree.generate_proof(1).unwrap();

        assert!(HashedLeanIMT::<SampleHasher>::verify_proof(&proof));

        assert_eq!(proof.index, 1);
        assert_eq!(&proof.leaf, b"leaf2");
    }

    #[test]
    fn test_index_of_and_get_leaf() {
        let leaves = vec![b"leaf1".to_vec(), b"leaf2".to_vec(), b"leaf3".to_vec()];

        let tree = HashedLeanIMT::new(&leaves, SampleHasher).unwrap();

        assert_eq!(tree.index_of(b"leaf2"), Some(1));
        assert_eq!(tree.index_of(b"nonexistent"), None);

        assert_eq!(tree.get_leaf(1), Some(b"leaf2".to_vec()));
        assert_eq!(tree.get_leaf(5), None);
    }
}
