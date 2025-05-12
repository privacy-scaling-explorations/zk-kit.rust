//! # Hashed LeanIMT
//!
//! Lean Incremental Merkle Tree hashing function wrapper.

use crate::lean_imt::{LeanIMT, LeanIMTError, MerkleProof};

/// LeanIMT hasher trait.
pub trait LeanIMTHasher<const N: usize> {
    fn hash(input: &[u8]) -> [u8; N];
}

/// Hashed LeanIMT struct.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HashedLeanIMT<const N: usize, H> {
    /// LeanIMT.
    tree: LeanIMT<N>,
    /// Hasher function.
    hasher: H,
}

impl<const N: usize, H> HashedLeanIMT<N, H>
where
    H: LeanIMTHasher<N>,
{
    /// Creates a new tree with optional initial leaves.
    pub fn new(leaves: &[[u8; N]], hasher: H) -> Result<Self, LeanIMTError> {
        let imt = LeanIMT::new(leaves, H::hash)?;

        Ok(Self { tree: imt, hasher })
    }

    /// Creates a new tree from a LeanIMT.
    pub fn new_from_tree(tree: LeanIMT<N>, hasher: H) -> Self {
        Self { tree, hasher }
    }

    /// Inserts a single leaf.
    pub fn insert(&mut self, leaf: &[u8; N]) {
        self.tree.insert(leaf, H::hash)
    }

    /// Inserts multiple leaves.
    ///
    /// # Errors
    /// 
    /// Will return [`LeanIMTError::EmptyBatchInsert`] if `leaves` is an empty array
    pub fn insert_many(&mut self, leaves: &[[u8; N]]) -> Result<(), LeanIMTError> {
        self.tree.insert_many(leaves, H::hash)
    }

    /// Updates a leaf at the given index.
    pub fn update(&mut self, index: usize, new_leaf: &[u8; N]) -> Result<(), LeanIMTError> {
        self.tree.update(index, new_leaf, H::hash)
    }

    /// Generates a Merkle proof for a leaf at the given index.
    pub fn generate_proof(&self, index: usize) -> Result<MerkleProof<N>, LeanIMTError> {
        self.tree.generate_proof(index)
    }

    /// Verifies a Merkle proof.
    pub fn verify_proof(proof: &MerkleProof<N>) -> bool {
        LeanIMT::verify_proof(proof, H::hash)
    }

    /// Returns the leaves.
    pub fn leaves(&self) -> &[[u8; N]] {
        self.tree.leaves()
    }

    /// Returns the number of leaves in the tree.
    pub fn size(&self) -> usize {
        self.tree.size()
    }

    /// Returns the tree root, if it exists.
    pub fn root(&self) -> Option<[u8; N]> {
        self.tree.root()
    }

    /// Returns the tree depth.
    pub fn depth(&self) -> usize {
        self.tree.depth()
    }

    /// Retrieves a leaf at the given index.
    pub fn get_leaf(&self, index: usize) -> Result<[u8; N], LeanIMTError> {
        self.tree.get_leaf(index)
    }

    /// Retrieves the node at a specified level and index.
    pub fn get_node(&self, level: usize, index: usize) -> Result<[u8; N], LeanIMTError> {
        self.tree.get_node(level, index)
    }

    /// Finds the index of a given leaf, if it exists.
    pub fn index_of(&self, leaf: &[u8]) -> Option<usize> {
        self.tree.index_of(leaf)
    }

    /// Checks whether the tree contains the specified leaf.
    pub fn contains(&self, leaf: &[u8]) -> bool {
        self.tree.contains(leaf)
    }

    /// Returns the tree.
    pub fn tree(&self) -> &LeanIMT<N> {
        &self.tree
    }

    /// Returns the hasher.
    pub fn hasher(&self) -> &H {
        &self.hasher
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    struct SampleHasher;

    impl LeanIMTHasher<32> for SampleHasher {
        fn hash(input: &[u8]) -> [u8; 32] {
            let mut hasher = DefaultHasher::new();
            input.hash(&mut hasher);
            let h = hasher.finish();
            let mut result = [0u8; 32];
            result[..8].copy_from_slice(&h.to_le_bytes());
            result
        }
    }

    #[test]
    fn test_new_empty_tree() {
        let tree = HashedLeanIMT::<32, SampleHasher>::new(&[], SampleHasher).unwrap();

        assert_eq!(tree.size(), 0);
        assert_eq!(tree.root(), None);
    }

    #[test]
    fn test_insert_leaves() {
        let mut tree = HashedLeanIMT::<32, SampleHasher>::new(&[], SampleHasher).unwrap();
        let leaf1 = [1u8; 32];

        tree.insert(&leaf1);

        assert_eq!(tree.size(), 1);
        assert!(tree.contains(&leaf1));

        let leaf2 = [2u8; 32];
        let leaf3 = [3u8; 32];

        tree.insert_many(&[leaf2, leaf3]).unwrap();

        assert_eq!(tree.size(), 3);
        assert!(tree.contains(&leaf2));
        assert!(tree.contains(&leaf3));
    }

    #[test]
    fn test_update_leaf() {
        let initial_leaves = [[0u8; 32], [1u8; 32]];
        let mut tree =
            HashedLeanIMT::<32, SampleHasher>::new(&initial_leaves, SampleHasher).unwrap();
        let updated_leaf = [42u8; 32];

        tree.update(0, &updated_leaf).unwrap();

        assert!(!tree.contains(&[0u8; 32]));
        assert!(tree.contains(&updated_leaf));

        assert!(tree.contains(&[1u8; 32]));
    }

    #[test]
    fn test_merkle_proof() {
        let leaves = vec![[0u8; 32], [1u8; 32], [2u8; 32], [3u8; 32]];
        let tree = HashedLeanIMT::<32, SampleHasher>::new(&leaves, SampleHasher).unwrap();
        let proof = tree.generate_proof(1).unwrap();

        assert!(HashedLeanIMT::<32, SampleHasher>::verify_proof(&proof));
        assert_eq!(proof.index, 1);
        assert_eq!(proof.leaf, [1u8; 32]);
    }

    #[test]
    fn test_index_of_and_get_leaf() {
        let leaves = vec![[0u8; 32], [1u8; 32], [2u8; 32]];
        let tree = HashedLeanIMT::<32, SampleHasher>::new(&leaves, SampleHasher).unwrap();

        assert_eq!(tree.index_of(&[1u8; 32]), Some(1));
        assert_eq!(tree.index_of(&[42u8; 32]), None);

        assert_eq!(tree.get_leaf(1), Ok([1u8; 32]));
        assert_eq!(tree.get_leaf(5), Err(LeanIMTError::IndexOutOfBounds));
    }
}
