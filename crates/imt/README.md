<p align="center">
    <h1 align="center">
       `zk-kit-imt`
    </h1>
    <p align="center">Incremental Merkle tree implementation in Rust.</p>
</p>

<p align="center">
    <a href="https://github.com/privacy-scaling-explorations/zk-kit">
        <img src="https://img.shields.io/badge/project-zk--kit-blue.svg?style=flat-square">
    </a>
    <a href="https://github.com/privacy-scaling-explorations/zk-kit.rust/tree/main/packages/imt/LICENSE">
        <img alt="License" src="https://img.shields.io/crates/l/zk-kit-imt?style=flat-square">
    </a>
    <a href="https://crates.io/crates/zk-kit-imt">
        <img alt="Version" src="https://img.shields.io/crates/v/zk-kit-imt?style=flat-square" />
    </a>
    <a href="https://crates.io/crates/zk-kit-imt">
        <img alt="Downloads" src="https://img.shields.io/crates/d/zk-kit-imt?style=flat-square" />
    </a>
</p>

<div align="center">
    <h4>
        <a href="https://appliedzkp.org/discord">
            🗣️ Chat &amp; Support
        </a>
    </h4>
</div>

In this implementation, the tree is built with a predetermined depth, utilizing a list of zeros (one for each level) to hash nodes lacking fully defined children. The tree's branching factor, or the number of children per node, can be customized via the arity parameter. For detailed insights into the implementation specifics, please refer to the [technical documentation](https://privacy-scaling-explorations.github.io/zk-kit.rust/zk_kit_imt/index.html).

---

## 🛠 Install

Install the `zk-kit-imt` crate with `cargo`:

```commandline
cargo add zk-kit-imt
```

## 📜 Usage

```rust
use zk_kit_imt::imt::IMT;

fn hash_function(nodes: Vec<String>) -> String {
    nodes.join("-")
}

fn main() {
    const ZERO: &str = "zero";
    const DEPTH: usize = 3;
    const ARITY: usize = 2;

    /*
     *  To create an instance of an IMT, you need to provide a hash function,
     *  the depth of the tree, the zero value, the arity of the tree and an initial list of leaves.
     */
    let mut tree = IMT::new(hash_function, DEPTH, ZERO.to_string(), ARITY, vec![]).unwrap();

    // Insert (incrementally) a leaf with value "some-leaf"
    tree.insert("some-leaf".to_string()).unwrap();
    // Insert (incrementally) a leaf with value "another_leaf"
    tree.insert("another_leaf".to_string()).unwrap();

    let root = tree.root().unwrap();
    println!("imt tree root: {root}");
    assert!(root == "some-leaf-another_leaf-zero-zero-zero-zero-zero-zero");

    let depth = tree.depth();
    println!("imt tree depth: {depth}");
    assert!(depth == 3);

    let arity = tree.arity();
    println!("imt tree arity: {arity}");
    assert!(arity == 2);

    let leaves = tree.leaves();
    println!("imt tree leaves: {:?}", leaves);
    assert!(leaves == vec!["some-leaf", "another_leaf"]);

    // Delete the leaf at index 0
    assert!(tree.delete(0).is_ok());
    let root = tree.root().unwrap();
    println!("imt tree root: {root}");
    assert!(root == "zero-another_leaf-zero-zero-zero-zero-zero-zero");

    // Create a proof for the leaf at index 1
    let proof = tree.create_proof(1);
    assert!(proof.is_ok());
    let proof = proof.unwrap();
    assert!(tree.verify_proof(&proof));
}
```
