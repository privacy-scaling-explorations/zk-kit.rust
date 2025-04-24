//! # Import/Export tests

#![cfg(feature = "serde")]

use lean_imt::{
    hashed_tree::{HashedLeanIMT, LeanIMTHasher},
    lean_imt::LeanIMT,
};
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
fn test_export_import_leanimt() {
    let leaves: Vec<[u8; 32]> = vec![[0u8; 32], [1u8; 32], [2u8; 32]];
    let tree = LeanIMT::new(&leaves, SampleHasher::hash).unwrap();

    let json = serde_json::to_string(&tree).unwrap();

    let imported_tree: LeanIMT<32> = serde_json::from_str(&json).unwrap();

    assert_eq!(tree, imported_tree);
}

#[test]
fn test_export_import_hashed_tree() {
    let leaves: Vec<[u8; 32]> = vec![[0u8; 32], [1u8; 32], [2u8; 32]];
    let hashed_tree = HashedLeanIMT::new(&leaves, SampleHasher).unwrap();

    let json = serde_json::to_string(&hashed_tree.tree()).unwrap();
    let imported_tree: LeanIMT<32> = serde_json::from_str(&json).unwrap();

    let hashed_tree_imported = HashedLeanIMT::new_from_tree(imported_tree, SampleHasher);

    assert_eq!(hashed_tree, hashed_tree_imported);
}
