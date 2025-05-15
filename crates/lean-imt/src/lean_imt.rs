//! # LeanIMT
//!
//! Lean Incremental Merkle Tree implementation.
//!
//! Specifications can be found here:
//!  - <https://github.com/privacy-scaling-explorations/zk-kit/blob/main/papers/leanimt/paper/leanimt-paper.pdf>

#![allow(clippy::manual_div_ceil)]

use thiserror::Error;

/// LeanIMT struct.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "[u8; N]: serde::Serialize",
        deserialize = "[u8; N]: serde::Deserialize<'de>"
    ))
)]
pub struct LeanIMT<const N: usize> {
    /// Nodes storage.
    nodes: Vec<Vec<[u8; N]>>,
}

impl<const N: usize> Default for LeanIMT<N> {
    fn default() -> Self {
        Self {
            nodes: vec![Vec::new()],
        }
    }
}

impl<const N: usize> LeanIMT<N> {
    /// Creates a new tree with optional initial leaves.
    pub fn new(leaves: &[[u8; N]], hash: impl Fn(&[u8]) -> [u8; N]) -> Result<Self, LeanIMTError> {
        let mut imt = Self::default();

        match leaves.len() {
            0 => {},
            1 => imt.insert(&leaves[0], hash),
            _ => imt.insert_many(leaves, hash)?,
        }

        Ok(imt)
    }

    /// Inserts a single leaf.
    pub fn insert(&mut self, leaf: &[u8; N], hash: impl Fn(&[u8]) -> [u8; N]) {
        let mut depth = self.depth();

        // Expand capacity if exceeded.
        if self.size() + 1 > (1 << depth) {
            self.nodes.push(Vec::new());
            depth += 1;
        }

        let mut node = *leaf;
        let mut index = self.size();

        for level in &mut self.nodes {
            // If the level is smaller than the expected index, we push a node
            if level.len() <= index {
                level.push(node);
            } else {
                level[index] = node;
            }

            // If we are at an odd index, we hash the leaves.
            if index % 2 == 1 {
                let mut hash_input = Vec::with_capacity(N * 2);

                // Sibling goes first.
                hash_input.extend_from_slice(&level[index - 1]);
                hash_input.extend_from_slice(&node);

                node = hash(&hash_input);
            }

            // Divide the expected index by 2.
            index >>= 1;
        }

        self.nodes[depth] = vec![node];
    }

    /// Inserts multiple leaves.
    pub fn insert_many(
        &mut self,
        leaves: &[[u8; N]],
        hash: impl Fn(&[u8]) -> [u8; N],
    ) -> Result<(), LeanIMTError> {
        if leaves.is_empty() {
            return Err(LeanIMTError::EmptyBatchInsert);
        }

        let start_index = self.size();
        self.nodes[0].extend_from_slice(leaves);

        // Ensure the tree has enough levels
        let required_depth = self.size().next_power_of_two().trailing_zeros() as usize;
        while self.depth() < required_depth {
            self.nodes.push(Vec::new());
        }

        // Start from level 0 and update parent nodes
        let mut index = start_index / 2;
        for level in 0..self.depth() {
            let level_len = self.nodes[level].len();
            let start_parent_idx = index;
            let num_parents = (level_len + 1) / 2;

            // Process each parent node starting from the affected index
            for parent_idx in start_parent_idx..num_parents {
                let left_idx = parent_idx * 2;
                let left = self.nodes[level][left_idx];

                let parent = if left_idx + 1 < level_len {
                    // Node has both children, hash them
                    let right = self.nodes[level][left_idx + 1];

                    let mut hash_input = Vec::with_capacity(2 * N);
                    hash_input.extend_from_slice(&left);
                    hash_input.extend_from_slice(&right);
                    hash(&hash_input)
                } else {
                    // Node has only left child, propagate it
                    left
                };

                // Update or add parent node
                let next_level = &mut self.nodes[level + 1];
                if parent_idx < next_level.len() {
                    next_level[parent_idx] = parent;
                } else {
                    next_level.push(parent);
                }
            }

            // Update index for the next level
            index /= 2;
        }

        Ok(())
    }

    /// Updates a leaf at the given index.
    pub fn update(
        &mut self,
        mut index: usize,
        new_leaf: &[u8; N],
        hash: impl Fn(&[u8]) -> [u8; N],
    ) -> Result<(), LeanIMTError> {
        if index >= self.size() {
            return Err(LeanIMTError::IndexOutOfBounds);
        }

        let mut node = *new_leaf;

        let depth = self.depth();
        for level in 0..depth {
            self.nodes[level][index] = node;
            if index & 1 != 0 {
                let sibling = self.nodes[level][index - 1];
                let mut hash_input = Vec::with_capacity(N * 2);
                hash_input.extend_from_slice(&sibling);
                hash_input.extend_from_slice(&node);
                node = hash(&hash_input);
            } else if let Some(sibling) = self.nodes[level].get(index + 1).copied() {
                let mut hash_input = Vec::with_capacity(N * 2);
                hash_input.extend_from_slice(&node);
                hash_input.extend_from_slice(&sibling);
                node = hash(&hash_input);
            }
            index >>= 1;
        }

        self.nodes[depth][0] = node;
        Ok(())
    }

    /// Generates a Merkle proof for a leaf at the given index.
    pub fn generate_proof(&self, mut index: usize) -> Result<MerkleProof<N>, LeanIMTError> {
        if index >= self.size() {
            return Err(LeanIMTError::IndexOutOfBounds);
        }

        let leaf = self.leaves()[index];
        let mut siblings = Vec::new();
        let mut path = Vec::new();

        for level in 0..self.depth() {
            let is_right = index & 1 != 0;
            let sibling_idx = if is_right { index - 1 } else { index + 1 };

            if let Some(sibling) = self.nodes[level].get(sibling_idx).copied() {
                path.push(is_right);
                siblings.push(sibling);
            }

            index >>= 1;
        }

        let final_index = path
            .iter()
            .rev()
            .fold(0, |acc, &is_right| (acc << 1) | is_right as usize);

        Ok(MerkleProof {
            root: self.nodes[self.depth()][0],
            leaf,
            index: final_index,
            siblings,
        })
    }

    /// Verifies a Merkle proof.
    pub fn verify_proof(proof: &MerkleProof<N>, hash: impl Fn(&[u8]) -> [u8; N]) -> bool {
        let mut node = proof.leaf;

        for (i, sibling) in proof.siblings.iter().enumerate() {
            let mut hash_input = Vec::with_capacity(N * 2);

            if (proof.index >> i) & 1 != 0 {
                // Right node
                hash_input.extend_from_slice(sibling);
                hash_input.extend_from_slice(&node);
            } else {
                // Left node
                hash_input.extend_from_slice(&node);
                hash_input.extend_from_slice(sibling);
            }

            node = hash(&hash_input);
        }

        proof.root == node
    }

    /// Returns the leaves.
    pub fn leaves(&self) -> &[[u8; N]] {
        if self.nodes.is_empty() {
            &[]
        } else {
            &self.nodes[0]
        }
    }

    /// Returns the number of leaves in the tree.
    pub fn size(&self) -> usize {
        self.leaves().len()
    }

    /// Returns the tree root, if it exists.
    pub fn root(&self) -> Option<[u8; N]> {
        self.nodes.last()?.first().copied()
    }

    /// Returns the tree depth.
    pub fn depth(&self) -> usize {
        self.nodes.len().saturating_sub(1)
    }

    /// Retrieves a leaf at the given index.
    pub fn get_leaf(&self, index: usize) -> Result<[u8; N], LeanIMTError> {
        self.leaves()
            .get(index)
            .copied()
            .ok_or(LeanIMTError::IndexOutOfBounds)
    }

    /// Returns the internal nodes structure.
    pub fn nodes(&self) -> &[Vec<[u8; N]>] {
        &self.nodes
    }

    /// Retrieves the node at a specified level and index.
    pub fn get_node(&self, level: usize, index: usize) -> Result<[u8; N], LeanIMTError> {
        let level_vec = self
            .nodes
            .get(level)
            .ok_or(LeanIMTError::LevelOutOfBounds)?;

        level_vec
            .get(index)
            .copied()
            .ok_or(LeanIMTError::IndexOutOfBounds)
    }

    /// Finds the index of a given leaf, if it exists.
    pub fn index_of(&self, leaf: &[u8]) -> Option<usize> {
        self.leaves().iter().position(|x| x == leaf)
    }

    /// Checks whether the tree contains the specified leaf.
    pub fn contains(&self, leaf: &[u8]) -> bool {
        self.index_of(leaf).is_some()
    }
}

/// Merkle proof.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "[u8; N]: serde::Serialize",
        deserialize = "[u8; N]: serde::Deserialize<'de>"
    ))
)]
pub struct MerkleProof<const N: usize> {
    /// Tree root.
    pub root: [u8; N],
    /// Leaf.
    pub leaf: [u8; N],
    /// Decimal representation of the reverse of the path.
    pub index: usize,
    /// Siblings.
    pub siblings: Vec<[u8; N]>,
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum LeanIMTError {
    #[error("Index out of bounds")]
    IndexOutOfBounds,
    #[error("Invalid leaf size")]
    InvalidLeafSize,
    #[error("Level out of bounds")]
    LevelOutOfBounds,
    #[error("Empty batch insert")]
    EmptyBatchInsert,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash(input: &[u8]) -> [u8; 4] {
        let mut hasher = DefaultHasher::new();

        for byte in input {
            byte.hash(&mut hasher);
        }
        let hash = hasher.finish();

        let mut result = [0u8; 4];
        result.copy_from_slice(&hash.to_le_bytes()[..4]);
        result
    }

    /// Convert a u32 into a [u8; 4]
    fn u32_to_leaf(n: u32) -> [u8; 4] {
        n.to_le_bytes()
    }

    /// Helper function to generate a vector of leaves from 0 to size - 1.
    fn generate_leaves(size: u32) -> Vec<[u8; 4]> {
        (0..size).map(u32_to_leaf).collect()
    }

    #[test]
    fn test_new_tree_empty() {
        let leaves: Vec<[u8; 4]> = vec![];
        let tree = LeanIMT::new(&leaves, hash).unwrap();

        assert_eq!(tree.size(), 0);
        assert_eq!(tree.root(), None);
        assert_eq!(tree.depth(), 0);

        let leaves: &[[u8; 4]] = tree.leaves();
        let empty_leaves: &[[u8; 4]] = &[];
        assert_eq!(leaves, empty_leaves);
    }

    #[test]
    fn test_insert_single_leaf() {
        let mut tree = LeanIMT::new(&[], hash).unwrap();
        let leaf = u32_to_leaf(1);
        tree.insert(&leaf, hash);

        assert_eq!(tree.root(), Some(leaf));
        assert_eq!(tree.size(), 1);
    }

    #[test]
    fn test_insert_multiple_leaves() {
        let leaves = generate_leaves(5);
        let tree_from_batch = LeanIMT::new(&leaves, hash).unwrap();

        // Create an empty tree and insert leaves one by one.
        let mut tree_iter = LeanIMT::new(&[], hash).unwrap();
        for leaf in leaves.iter() {
            tree_iter.insert(leaf, hash);
        }

        assert_eq!(tree_from_batch, tree_iter);
    }

    #[test]
    fn test_index_of_and_contains() {
        let leaves = generate_leaves(5);
        let tree = LeanIMT::new(&leaves, hash).unwrap();

        assert_eq!(tree.index_of(&u32_to_leaf(2)), Some(2));
        assert!(tree.contains(&u32_to_leaf(2)));

        assert_eq!(tree.index_of(&u32_to_leaf(999)), None);
        assert!(!tree.contains(&u32_to_leaf(999)));
    }

    #[test]
    fn test_update_leaf() {
        let leaves = generate_leaves(5);
        let mut tree = LeanIMT::new(&leaves, hash).unwrap();

        let new_leaf = u32_to_leaf(42);
        tree.update(0, &new_leaf, hash).unwrap();
        assert_eq!(tree.get_leaf(0).unwrap(), new_leaf);

        let proof = tree.generate_proof(0).unwrap();
        assert!(LeanIMT::verify_proof(&proof, hash));
    }

    #[test]
    fn test_generate_and_verify_proof() {
        let leaves = generate_leaves(5);
        let tree = LeanIMT::new(&leaves, hash).unwrap();

        for i in 0..leaves.len() {
            let proof = tree.generate_proof(i).unwrap();
            assert_eq!(proof.leaf, leaves[i]);
            assert_eq!(proof.root, tree.root().unwrap());
            assert!(LeanIMT::verify_proof(&proof, hash));
        }
    }

    #[test]
    fn test_generate_proof_invalid_index() {
        let leaves = generate_leaves(5);
        let tree = LeanIMT::new(&leaves, hash).unwrap();

        let result = tree.generate_proof(999);
        assert!(matches!(result, Err(LeanIMTError::IndexOutOfBounds)));
    }

    #[test]
    fn test_update_invalid_index() {
        let leaves = generate_leaves(5);
        let mut tree = LeanIMT::new(&leaves, hash).unwrap();

        let result = tree.update(100, &u32_to_leaf(10), hash);
        assert!(matches!(result, Err(LeanIMTError::IndexOutOfBounds)));
    }
}
