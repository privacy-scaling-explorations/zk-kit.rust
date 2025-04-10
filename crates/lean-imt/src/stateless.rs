//! Stateless proofs module.
//!
//! This is a feature module that provides functionalities to generate inclusion proofs without the tree data.

use crate::lean_imt::LeanIMTError;

/// Stateless proof element
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProofElement {
    /// Tree level
    level: usize,
    /// Sibling index
    sibling_index: usize,
}

impl ProofElement {
    /// Creates a new `ProofElement` with the given level and sibling index.
    pub fn new(level: usize, sibling_index: usize) -> Self {
        Self {
            level,
            sibling_index,
        }
    }

    /// Returns the level of the proof element.
    pub fn level(&self) -> usize {
        self.level
    }

    /// Returns the sibling index of the proof element.
    pub fn sibling_index(&self) -> usize {
        self.sibling_index
    }

    /// Determines if the original node was a right child.
    /// If the sibling index is even, the original node was a right child.
    pub fn is_right(&self) -> bool {
        self.sibling_index & 1 == 0
    }
}

/// Computes the set of elements of a Merkle proof of a Lean IMT without the tree data.
///
/// It receives the index of the leaf we want to prove and the tree size.
///
/// The set it returns will not contain the actual leaf data but their positions in the tree.
pub fn stateless_path(
    leaf_index: usize,
    tree_size: usize,
) -> Result<Vec<ProofElement>, LeanIMTError> {
    if leaf_index >= tree_size {
        return Err(LeanIMTError::IndexOutOfBounds);
    }

    let depth = tree_size.next_power_of_two().trailing_zeros() as usize;
    let mut proof_elements = Vec::with_capacity(depth);
    let (mut current_index, mut current_level_size) = (leaf_index, tree_size);

    for level in 0..depth {
        let is_right = (current_index & 1) != 0;
        let sibling = if is_right {
            current_index - 1
        } else {
            current_index + 1
        };

        if sibling < current_level_size {
            proof_elements.push(ProofElement::new(level, sibling));
        }

        current_index /= 2;
        current_level_size = (current_level_size + 1) / 2;
    }

    Ok(proof_elements)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hashed_tree::{HashedLeanIMT, LeanIMTHasher},
        lean_imt::MerkleProof,
    };
    use rand::Rng;
    use std::hash::{DefaultHasher, Hash, Hasher};

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

    /// Reconstructs a Merkle proof from tree data and a slice of proof elements.
    fn proof_from_indices(
        tree: &HashedLeanIMT<32, SampleHasher>,
        leaf_index: usize,
        proof_elements: &[ProofElement],
    ) -> MerkleProof<32> {
        let mut siblings = Vec::with_capacity(proof_elements.len());
        let mut bits = Vec::with_capacity(proof_elements.len());

        for elem in proof_elements.iter() {
            siblings.push(tree.get_node(elem.level(), elem.sibling_index()).unwrap());
            bits.push(if elem.is_right() { 1 } else { 0 });
        }

        let mut encoded_index = 0;
        for bit in bits.into_iter().rev() {
            encoded_index = (encoded_index << 1) | bit;
        }

        MerkleProof {
            root: tree.root().unwrap(),
            leaf: tree.get_leaf(leaf_index).unwrap(),
            siblings,
            index: encoded_index,
        }
    }

    // Helper function to create a random leaf
    fn random_leaf(rng: &mut impl Rng) -> [u8; 32] {
        let mut leaf = [0u8; 32];
        rng.fill(&mut leaf);
        leaf
    }

    #[test]
    fn test_stateless_merkle_path() {
        let mut rng = rand::rng();
        let max_tree_size = 2_usize.pow(10);

        let mut random_leaf_set = Vec::with_capacity(max_tree_size);
        for _ in 0..max_tree_size {
            random_leaf_set.push(random_leaf(&mut rng));
        }

        for size in 0..max_tree_size {
            let tree =
                HashedLeanIMT::<32, SampleHasher>::new(&random_leaf_set[0..size], SampleHasher)
                    .unwrap();

            for leaf_index in 0..size {
                let stateless_indices = stateless_path(leaf_index, size).unwrap();
                let stateless_proof = proof_from_indices(&tree, leaf_index, &stateless_indices);
                let actual_proof = tree.generate_proof(leaf_index).unwrap();
                assert_eq!(actual_proof, stateless_proof);
            }
        }
    }
}
