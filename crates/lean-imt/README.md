<p align="center">
    <h1 align="center">
       LeanIMT
    </h1>
    <p align="center">Lean Incremental Merkle Tree implementation in Rust.</p>
</p>

<p align="center">
    <a href="https://github.com/privacy-scaling-explorations/zk-kit">
        <img src="https://img.shields.io/badge/project-zk--kit-blue.svg?style=flat-square">
    </a>
    <a href="https://github.com/privacy-scaling-explorations/zk-kit.rust/tree/main/LICENSE">
        <img alt="License" src="https://img.shields.io/badge/license-MIT-blue.svg">
    </a>
</p>

<div align="center">
    <h4>
        <a href="https://appliedzkp.org/discord">
            🗣️ Chat &amp; Support
        </a>
    </h4>
</div>

The LeanIMT is an optimized binary version of the [IMT](https://github.com/privacy-scaling-explorations/zk-kit/tree/main/packages/imt) into binary-focused model, eliminating the need for zero values and allowing dynamic depth adjustment. Unlike the IMT, which uses a zero hash for incomplete nodes, the LeanIMT directly adopts the left child's value when a node lacks a right counterpart. The tree's depth dynamically adjusts to the count of leaves, enhancing efficiency by reducing the number of required hash calculations. To understand more about the LeanIMT, take a look at this [visual explanation](https://hackmd.io/@vplasencia/S1whLBN16). For detailed insights into the implementation specifics, please refer to the [LeanIMT paper](https://github.com/privacy-scaling-explorations/zk-kit/tree/main/papers/leanimt).

---

## 🛠 Install

Add the `zk-kit-lean-imt` crate to your `cargo.toml`:

```toml
zk-kit-lean-imt = { git = "https://github.com/privacy-scaling-explorations/zk-kit.rust", package = "zk-kit-lean-imt" }
```

## 📜 Usage

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use zk_kit_lean_imt::hashed_tree::{HashedLeanIMT, LeanIMTHasher};

// Setup hasher
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

fn main() {
    // Create an empty tree
    let mut tree = HashedLeanIMT::<32, SampleHasher>::new(&[], SampleHasher).unwrap();

    // Insert individual leaves
    tree.insert(&[1; 32]);
    tree.insert(&[2; 32]);

    // Insert multiple leaves
    tree.insert_many(&[[3; 32], [4; 32], [5; 32]]).unwrap();

    // Get the root
    let root = tree.root().unwrap();
    println!("Tree root: {:?}", root);

    // Get the tree depth
    let depth = tree.depth();
    println!("Tree depth: {}", depth);

    // Generate a proof for the leaf at index 1
    let proof = tree.generate_proof(1).unwrap();

    // Verify the proof
    assert!(HashedLeanIMT::<32, SampleHasher>::verify_proof(&proof));
}
```
