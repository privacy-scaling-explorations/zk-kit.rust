//! # LeanIMT
//!
//! Lean Incremental Merkle Tree implementation.
//!
//! Specifications can be found here:
//!  - https://github.com/privacy-scaling-explorations/zk-kit/tree/main/papers/leanimt

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// LeanIMT Merkle proof.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct MerkleProof {
    /// Root of the tree.
    pub root: Vec<u8>,
    /// Leaf of the tree.
    pub leaf: Vec<u8>,
    /// Path to the leaf.
    pub siblings: Vec<Vec<u8>>,
    /// Leaf index.
    pub index: usize,
}

/// LeanIMT struct.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct LeanIMT {
    /// Nodes
    nodes: Vec<Vec<Vec<u8>>>,
}

impl LeanIMT {
    /// Creates a new tree with optional initial leaves.
    pub fn new(leaves: &[Vec<u8>], hash: impl Fn(&[u8]) -> Vec<u8>) -> Result<Self, LeanIMTError> {
        let mut imt = Self {
            nodes: vec![vec![]],
        };

        match leaves.len() {
            0 => {},
            1 => imt.insert(&leaves[0], hash),
            _ => imt.insert_many(leaves, hash)?,
        }

        Ok(imt)
    }

    /// Returns the root, if it exists.
    pub fn root(&self) -> Option<Vec<u8>> {
        self.nodes.last().and_then(|level| level.first()).cloned()
    }

    /// Returns the tree depth.
    pub fn depth(&self) -> usize {
        self.nodes.len() - 1
    }

    /// Returns the leaves.
    pub fn leaves(&self) -> &[Vec<u8>] {
        self.nodes[0].as_slice()
    }

    /// Returns the number of leaves.
    pub fn size(&self) -> usize {
        self.nodes[0].len()
    }

    /// Returns the index of a leaf, if it exists.
    pub fn index_of(&self, leaf: &[u8]) -> Option<usize> {
        self.leaves().iter().position(|x| x == leaf)
    }

    /// Checks if a leaf exists.
    pub fn contains(&self, leaf: &[u8]) -> bool {
        self.index_of(leaf).is_some()
    }

    /// Returns the leaf at the given index.
    pub fn get_leaf(&self, index: usize) -> Option<Vec<u8>> {
        self.leaves().get(index).cloned()
    }

    /// Inserts a single leaf.
    pub fn insert(&mut self, leaf: &[u8], hash: impl Fn(&[u8]) -> Vec<u8>) {
        let new_size = self.size() + 1;
        let new_depth = new_size.next_power_of_two().trailing_zeros() as usize;

        if self.depth() < new_depth {
            self.nodes.push(Vec::new());
        }

        let mut node = leaf.to_vec();
        let mut index = self.size();

        for level in 0..new_depth {
            if self.nodes[level].len() <= index {
                self.nodes[level].push(node.clone());
            } else {
                self.nodes[level][index] = node.clone();
            }

            if index & 1 != 0 {
                let sibling = &self.nodes[level][index - 1];
                let mut hash_input = sibling.clone();
                hash_input.extend(node.iter());
                node = hash(&hash_input);
            }

            index >>= 1;
        }

        self.nodes[new_depth] = vec![node];
    }

    /// Inserts multiple leaves.
    pub fn insert_many(
        &mut self,
        leaves: &[Vec<u8>],
        hash: impl Fn(&[u8]) -> Vec<u8>,
    ) -> Result<(), LeanIMTError> {
        if leaves.is_empty() {
            return Err(LeanIMTError::EmptyBatchInsert);
        }

        let mut start_index = self.size() >> 1;
        self.nodes[0].extend(leaves.iter().cloned());

        let new_size = self.size();
        let new_depth = new_size.next_power_of_two().trailing_zeros() as usize;
        let number_of_new_levels = new_depth - self.depth();

        for _ in 0..number_of_new_levels {
            self.nodes.push(Vec::new());
        }

        for level in 0..self.depth() {
            let number_of_nodes = (self.nodes[level].len() as f64 / 2.0).ceil() as usize;
            for index in start_index..number_of_nodes {
                let left_node = &self.nodes[level][index * 2];
                let parent_node = if index * 2 + 1 < self.nodes[level].len() {
                    let right_node = &self.nodes[level][index * 2 + 1];

                    let mut hash_input = left_node.clone();
                    hash_input.extend(right_node.iter());
                    hash(&hash_input)
                } else {
                    left_node.clone()
                };

                if self.nodes[level + 1].len() <= index {
                    self.nodes[level + 1].push(parent_node);
                } else {
                    self.nodes[level + 1][index] = parent_node;
                }
            }
            start_index >>= 1;
        }

        Ok(())
    }

    /// Updates a leaf at the given index.
    pub fn update(
        &mut self,
        mut index: usize,
        new_leaf: &[u8],
        hash: impl Fn(&[u8]) -> Vec<u8>,
    ) -> Result<(), LeanIMTError> {
        if index >= self.size() {
            return Err(LeanIMTError::IndexOutOfBounds);
        }

        let mut node = new_leaf.to_vec();

        let depth = self.depth();
        for level in 0..depth {
            self.nodes[level][index] = node.clone();
            if index & 1 != 0 {
                let sibling = &self.nodes[level][index - 1];

                let mut hash_input = sibling.clone();
                hash_input.extend(node.iter());
                node = hash(&hash_input);
            } else if let Some(sibling) = self.nodes[level].get(index + 1) {
                let mut hash_input = node.clone();
                hash_input.extend(sibling.iter());
                node = hash(&hash_input);
            }
            index >>= 1;
        }

        self.nodes[depth][0] = node;
        Ok(())
    }

    /// Generates a Merkle proof for a leaf at the given index.
    pub fn generate_proof(&self, mut index: usize) -> Result<MerkleProof, LeanIMTError> {
        if index >= self.size() {
            return Err(LeanIMTError::IndexOutOfBounds);
        }

        let leaf = self.leaves()[index].clone();
        let mut siblings = vec![];
        let mut path = vec![];

        for level in 0..self.depth() {
            let is_right_node = index & 1 != 0;
            let sibling_index = if is_right_node { index - 1 } else { index + 1 };

            if let Some(sibling) = self.nodes[level].get(sibling_index).cloned() {
                path.push(is_right_node);
                siblings.push(sibling);
            }

            index >>= 1;
        }

        let final_index = path
            .iter()
            .rev()
            .fold(0, |acc, &is_right| (acc << 1) | is_right as usize);

        Ok(MerkleProof {
            root: self.nodes[self.depth()][0].clone(),
            leaf,
            index: final_index,
            siblings,
        })
    }

    /// Verifies a Merkle proof.
    pub fn verify_proof(proof: &MerkleProof, hash: impl Fn(&[u8]) -> Vec<u8>) -> bool {
        let mut node = proof.leaf.to_vec();
        for (i, sibling) in proof.siblings.iter().enumerate() {
            node = if (proof.index >> i) & 1 != 0 {
                let mut hash_input = sibling.clone();
                hash_input.extend(node.iter());
                hash(&hash_input)
            } else {
                let mut hash_input = node.clone();
                hash_input.extend(sibling.iter());
                hash(&hash_input)
            };
        }
        proof.root == node
    }
}

#[derive(Error, Debug)]
pub enum LeanIMTError {
    #[error("Leaf index out of bounds")]
    IndexOutOfBounds,
    #[error("Batch insert with empty set of leaves")]
    EmptyBatchInsert,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn test_hash(data: &[u8]) -> Vec<u8> {
        let mut hasher = DefaultHasher::new();
        for byte in data {
            byte.hash(&mut hasher);
        }
        let hash = hasher.finish();
        let mut result = vec![0u8; 32];
        result[..8].copy_from_slice(&hash.to_le_bytes());
        result
    }

    fn create_test_leaf(value: u8) -> Vec<u8> {
        vec![value; 32]
    }

    #[test]
    fn new_empty() {
        let imt = LeanIMT::new(&[], test_hash).unwrap();
        assert_eq!(imt.size(), 0);
        assert_eq!(imt.depth(), 0);
        assert_eq!(imt.leaves(), &[] as &[Vec<u8>]);
        assert_eq!(imt.root(), None);
    }

    #[test]
    fn new_with_leaves() {
        let leaves = vec![
            create_test_leaf(0),
            create_test_leaf(1),
            create_test_leaf(2),
        ];
        let imt = LeanIMT::new(&leaves, test_hash).unwrap();
        assert_eq!(imt.size(), 3);
        assert_eq!(imt.depth(), 2);
        assert_eq!(imt.leaves(), leaves);
    }

    #[test]
    fn contains_and_index_of() {
        let leaf1 = create_test_leaf(0);
        let leaf2 = create_test_leaf(1);
        let leaf3 = create_test_leaf(2);
        let imt = LeanIMT::new(&[leaf1.clone(), leaf2.clone(), leaf3.clone()], test_hash).unwrap();

        assert_eq!(imt.index_of(&leaf2), Some(1));
        assert!(imt.contains(&leaf2));
        assert_eq!(imt.index_of(&create_test_leaf(3)), None);
        assert!(!imt.contains(&create_test_leaf(3)));
    }

    #[test]
    fn insert_single() {
        let mut imt = LeanIMT::new(&[], test_hash).unwrap();
        let leaf = create_test_leaf(0);
        imt.insert(&leaf, test_hash);
        assert_eq!(imt.size(), 1);
        assert_eq!(imt.depth(), 0);
        assert_eq!(imt.root(), Some(leaf));
    }

    #[test]
    fn insert_multiple() {
        let leaf1 = create_test_leaf(0);
        let leaf2 = create_test_leaf(1);
        let leaf3 = create_test_leaf(2);
        let leaf4 = create_test_leaf(3);

        let mut imt = LeanIMT::new(&[], test_hash).unwrap();
        imt.insert(&leaf1, test_hash);
        imt.insert(&leaf2, test_hash);
        imt.insert(&leaf3, test_hash);
        imt.insert(&leaf4, test_hash);

        assert_eq!(imt.size(), 4);
        assert_eq!(imt.depth(), 2);
    }

    #[test]
    fn insert_many_empty() {
        let mut imt = LeanIMT::new(&[], test_hash).unwrap();
        assert!(imt.insert_many(&[], test_hash).is_err());
    }

    #[test]
    fn insert_many_multiple() {
        let leaf1 = create_test_leaf(0);
        let leaf2 = create_test_leaf(1);
        let leaf3 = create_test_leaf(2);
        let leaf4 = create_test_leaf(3);

        let mut imt = LeanIMT::new(&[leaf1, leaf2], test_hash).unwrap();
        imt.insert_many(&[leaf3, leaf4], test_hash).unwrap();

        assert_eq!(imt.size(), 4);
        assert_eq!(imt.depth(), 2);
    }

    #[test]
    fn update_leaf() {
        let leaf1 = create_test_leaf(0);
        let leaf2 = create_test_leaf(1);
        let leaf3 = create_test_leaf(2);
        let leaf4 = create_test_leaf(3);
        let new_leaf = create_test_leaf(4);

        let mut imt = LeanIMT::new(&[leaf1, leaf2.clone(), leaf3, leaf4], test_hash).unwrap();
        imt.update(1, &new_leaf, test_hash).unwrap();

        assert_eq!(imt.size(), 4);
        assert_eq!(imt.leaves()[1], new_leaf);
    }

    #[test]
    fn update_out_of_bounds() {
        let mut imt = LeanIMT::new(&[create_test_leaf(0)], test_hash).unwrap();
        assert!(imt.update(1, &create_test_leaf(1), test_hash).is_err());
    }

    #[test]
    fn generate_proof() {
        let leaf1 = create_test_leaf(0);
        let leaf2 = create_test_leaf(1);
        let leaf3 = create_test_leaf(2);
        let leaf4 = create_test_leaf(3);

        let imt = LeanIMT::new(&[leaf1, leaf2, leaf3.clone(), leaf4], test_hash).unwrap();
        let proof = imt.generate_proof(2).unwrap();

        assert_eq!(proof.leaf, leaf3);
        assert_eq!(proof.index, 2);
    }

    #[test]
    fn generate_proof_out_of_bounds() {
        let imt = LeanIMT::new(&[create_test_leaf(0)], test_hash).unwrap();
        assert!(imt.generate_proof(1).is_err());
    }

    #[test]
    fn verify_proof() {
        let leaf1 = create_test_leaf(0);
        let leaf2 = create_test_leaf(1);
        let leaf3 = create_test_leaf(2);
        let leaf4 = create_test_leaf(3);

        let imt = LeanIMT::new(&[leaf1, leaf2, leaf3, leaf4], test_hash).unwrap();
        let proof = imt.generate_proof(2).unwrap();
        assert!(LeanIMT::verify_proof(&proof, test_hash));
    }
}
