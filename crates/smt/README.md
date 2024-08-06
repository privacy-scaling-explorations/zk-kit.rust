<p align="center">
    <h1 align="center">
        Sparse Merkle tree
    </h1>
    <p align="center">Sparse Merkle tree implementation in Rust.</p>
</p>

<p align="center">
    <a href="https://github.com/privacy-scaling-explorations/zk-kit">
        <img src="https://img.shields.io/badge/project-zk--kit-blue.svg?style=flat-square">
    </a>
    <a href="https://github.com/privacy-scaling-explorations/zk-kit.rust/tree/main/packages/smt/LICENSE">
        <img alt="License" src="https://img.shields.io/crates/l/zk-kit-smt?style=flat-square">
    </a>
    <a href="https://crates.io/crates/zk-kit-smt">
        <img alt="Version" src="https://img.shields.io/crates/v/zk-kit-smt?style=flat-square" />
    </a>
    <a href="https://crates.io/crates/zk-kit-smt">
        <img alt="Downloads" src="https://img.shields.io/crates/d/zk-kit-smt?style=flat-square" />
    </a>
</p>

<div align="center">
    <h4>
        <a href="https://appliedzkp.org/discord">
            🗣️ Chat &amp; Support
        </a>
    </h4>
</div>

A sparse Merkle tree is a data structure useful for storing a key/value map where every leaf node of the tree contains the cryptographic hash of a key/value pair and every non leaf node contains the concatenated hashes of its child nodes. Sparse Merkle trees provides a secure and efficient verification of large data sets and they are often used in peer-to-peer technologies. This implementation is an optimized version of the traditional sparse Merkle tree and it is based on the concepts expressed in the papers and resources below.

## References

1. Rasmus Dahlberg, Tobias Pulls and Roel Peeters. _Efficient Sparse Merkle Trees: Caching Strategies and Secure (Non-)Membership Proofs_. Cryptology ePrint Archive: Report 2016/683, 2016. https://eprint.iacr.org/2016/683.
2. Faraz Haider. _Compact sparse merkle trees_. Cryptology ePrint Archive: Report 2018/955, 2018. https://eprint.iacr.org/2018/955.
3. Jordi Baylina and Marta Bellés. _Sparse Merkle Trees_. https://docs.iden3.io/publications/pdfs/Merkle-Tree.pdf.
4. Vitalik Buterin Fichter. _Optimizing sparse Merkle trees_. https://ethresear.ch/t/optimizing-sparse-merkle-trees/3751.

---

## 🛠 Install

You can install `zk-kit-smt` crate with `cargo`:

```bash
cargo add zk-kit-smt
```

## 📜 Usage

```rust
use zk_kit_smt::smt::{Key, Node, Value, SMT};

fn hash_function(nodes: Vec<Node>) -> Node {
    let strings: Vec<String> = nodes.iter().map(|node| node.to_string()).collect();
    Node::Str(strings.join(","))
}

fn main() {
    // Initialize the Sparse Merkle Tree with a hash function.
    let mut smt = SMT::new(hash_function, false);

    let key = Key::Str("aaa".to_string());
    let value = Value::Str("bbb".to_string());

    // Add a key-value pair to the Sparse Merkle Tree.
    smt.add(key.clone(), value.clone()).unwrap();

    // Get the value of the key.
    let get = smt.get(key.clone());
    assert_eq!(get, Some(value));

    // Update the value of the key.
    let new_value = Value::Str("ccc".to_string());
    let update = smt.update(key.clone(), new_value.clone());
    assert!(update.is_ok());
    assert_eq!(smt.get(key.clone()), Some(new_value));

    // Create and verify a proof for the key.
    let create_proof = smt.create_proof(key.clone());
    let verify_proof = smt.verify_proof(create_proof);
    assert!(verify_proof);

    // Delete the key.
    let delete = smt.delete(key.clone());
    assert!(delete.is_ok());
    assert_eq!(smt.get(key.clone()), None);
}
```
