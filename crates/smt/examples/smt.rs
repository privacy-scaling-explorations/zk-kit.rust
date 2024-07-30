use zk_kit_smt::smt::{Key, Node, Value, SMT};

fn hash_function(nodes: Vec<Node>) -> Node {
    let strings: Vec<String> = nodes.iter().map(|node| node.to_string()).collect();
    Node::Str(strings.join(","))
}

fn main() {
    // Initialize the Sparse Merkle Tree with a hash function.
    let mut smt = SMT::new(hash_function, false);

    let key = Key::Str("abc".to_string());
    let value = Value::Str("123".to_string());

    // Add a key-value pair to the Sparse Merkle Tree.
    smt.add(key.clone(), value.clone()).unwrap();

    // Get the value of the key.
    let get = smt.get(key.clone());
    assert_eq!(get, Some(value));

    // Update the value of the key.
    let new_value = Value::Str("456".to_string());
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
