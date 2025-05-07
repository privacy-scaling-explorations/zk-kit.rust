use std::{collections::HashMap, str::FromStr};

use num_bigint::BigInt;

use crate::utils::{
    get_first_common_elements, get_index_of_last_non_zero_element, is_hexadecimal, key_to_path,
};

use std::fmt;

#[derive(Debug, PartialEq)]
pub enum SMTError {
    KeyAlreadyExist(String),
    KeyDoesNotExist(String),
    InvalidParameterType(String, String),
    InvalidSiblingIndex,
}

impl fmt::Display for SMTError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SMTError::KeyAlreadyExist(s) => write!(f, "Key {} already exists", s),
            SMTError::KeyDoesNotExist(s) => write!(f, "Key {} does not exist", s),
            SMTError::InvalidParameterType(p, t) => {
                write!(f, "Parameter {} must be a {}", p, t)
            },
            SMTError::InvalidSiblingIndex => write!(f, "Invalid sibling index"),
        }
    }
}

impl std::error::Error for SMTError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Node {
    Str(String),
    BigInt(BigInt),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Str(s) => write!(f, "{}", s),
            Node::BigInt(n) => write!(f, "{}", n),
        }
    }
}

impl FromStr for Node {
    type Err = SMTError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(bigint) = s.parse::<BigInt>() {
            Ok(Node::BigInt(bigint))
        } else if is_hexadecimal(s) {
            Ok(Node::Str(s.to_string()))
        } else {
            Err(SMTError::InvalidParameterType(
                s.to_string(),
                "BigInt or hexadecimal string".to_string(),
            ))
        }
    }
}

pub type Key = Node;
pub type Value = Node;
pub type EntryMark = Node;

pub type Entry = (Key, Value, EntryMark);
pub type ChildNodes = Vec<Node>;
pub type Siblings = Vec<Node>;

pub type HashFunction = fn(ChildNodes) -> Node;

pub struct EntryResponse {
    pub entry: Vec<Node>,
    pub matching_entry: Option<Vec<Node>>,
    pub siblings: Siblings,
}

#[allow(dead_code)]
pub struct MerkleProof {
    entry_response: EntryResponse,
    root: Node,
    membership: bool,
}

#[allow(dead_code)]
pub struct SMT {
    hash: HashFunction,
    big_numbers: bool,
    zero_node: Node,
    entry_mark: Node,
    nodes: HashMap<Node, Vec<Node>>,
    root: Node,
}

impl SMT {
    /// Initializes a new instance of the Sparse Merkle Tree (SMT).
    ///
    /// # Arguments
    ///
    /// * `hash` - The hash function used to hash the child nodes.
    /// * `big_numbers` - A flag indicating whether the SMT supports big numbers or not.
    ///
    /// # Returns
    ///
    /// A new instance of the SMT.
    pub fn new(hash: HashFunction, big_numbers: bool) -> Self {
        let zero_node;
        let entry_mark;

        if big_numbers {
            zero_node = Node::BigInt(BigInt::from(0));
            entry_mark = Node::BigInt(BigInt::from(1));
        } else {
            zero_node = Node::Str("0".to_string());
            entry_mark = Node::Str("1".to_string());
        }

        SMT {
            hash,
            big_numbers,
            zero_node: zero_node.clone(),
            entry_mark,
            nodes: HashMap::new(),
            root: zero_node,
        }
    }

    /// Retrieves the value associated with the given key from the SMT.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the value for.
    ///
    /// # Returns
    ///
    /// An `Option` containing the value associated with the key, or `None` if the key does not exist.
    pub fn get(&self, key: Key) -> Option<Value> {
        let key = key.to_string().parse::<Node>().unwrap();

        let EntryResponse { entry, .. } = self.retrieve_entry(key);

        entry.get(1).cloned()
    }

    /// Adds a new key-value pair to the SMT.
    ///
    /// It retrieves a matching entry or a zero node with a top-down approach and then it updates
    /// all the hashes of the nodes in the path of the new entry with a bottom up approach.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to add.
    /// * `value` - The value associated with the key.
    ///
    /// # Returns
    ///
    /// An `Result` indicating whether the operation was successful or not.
    pub fn add(&mut self, key: Key, value: Value) -> Result<(), SMTError> {
        let key = key.to_string().parse::<Node>().unwrap();
        let value = value.to_string().parse::<Node>().unwrap();

        let EntryResponse {
            entry,
            matching_entry,
            mut siblings,
        } = self.retrieve_entry(key.clone());

        if entry.get(1).is_some() {
            return Err(SMTError::KeyAlreadyExist(key.to_string()));
        }

        let path = key_to_path(&key.to_string());
        // If there is a matching entry, its node is saved in the `node` variable, otherwise the
        // `zero_node` is saved. This node is used below as the first node (starting from the
        // bottom of the tree) to obtain the new nodes up to the root.
        let node = if let Some(ref matching_entry) = matching_entry {
            (self.hash)(matching_entry.clone())
        } else {
            self.zero_node.clone()
        };

        // If there are siblings, the old nodes are deleted and will be re-created below with new hashes.
        if !siblings.is_empty() {
            self.delete_old_nodes(node.clone(), &path, &siblings)
        }

        // If there is a matching entry, further N zero siblings are added in the `siblings` vector,
        // followed by the matching node itself. N is the number of the first matching bits of the paths.
        // This is helpful in the non-membership proof verification as explained in the function below.
        if let Some(matching_entry) = matching_entry {
            let matching_path = key_to_path(&matching_entry[0].to_string());
            let mut i = siblings.len();

            while matching_path[i] == path[i] {
                siblings.push(self.zero_node.clone());
                i += 1;
            }

            siblings.push(node.clone());
        }

        // Adds the new entry and re-creates the nodes of the path with the new hashes with a bottom
        // up approach. The `add_new_nodes` function returns the new root of the tree.
        let new_node = (self.hash)(vec![key.clone(), value.clone(), self.entry_mark.clone()]);

        self.nodes
            .insert(new_node.clone(), vec![key, value, self.entry_mark.clone()]);
        self.root = self
            .add_new_nodes(new_node, &path, &siblings, None)
            .unwrap();

        Ok(())
    }

    /// Updates the value associated with the given key in the SMT.
    ///
    /// Also in this case, all the hashes of the nodes in the path of the updated entry are updated
    /// with a bottom up approach.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to update the value for.
    /// * `value` - The new value associated with the key.
    ///
    /// # Returns
    ///
    /// An `Result` indicating whether the operation was successful or not.
    pub fn update(&mut self, key: Key, value: Value) -> Result<(), SMTError> {
        let key = key.to_string().parse::<Node>().unwrap();
        let value = value.to_string().parse::<Node>().unwrap();

        let EntryResponse {
            entry, siblings, ..
        } = self.retrieve_entry(key.clone());

        if entry.get(1).is_none() {
            return Err(SMTError::KeyDoesNotExist(key.to_string()));
        }

        let path = key_to_path(&key.to_string());

        // Deletes the old nodes and re-creates them with the new hashes.
        let old_node = (self.hash)(entry.clone());
        self.nodes.remove(&old_node);
        self.delete_old_nodes(old_node.clone(), &path, &siblings);

        let new_node = (self.hash)(vec![key.clone(), value.clone(), self.entry_mark.clone()]);
        self.nodes
            .insert(new_node.clone(), vec![key, value, self.entry_mark.clone()]);
        self.root = self
            .add_new_nodes(new_node, &path, &siblings, None)
            .unwrap();

        Ok(())
    }

    /// Deletes the key-value pair associated with the given key from the SMT.
    ///
    /// Also in this case, all the hashes of the nodes in the path of the deleted entry are updated
    /// with a bottom up approach.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to delete.
    ///
    /// # Returns
    ///
    /// An `Result` indicating whether the operation was successful or not.
    pub fn delete(&mut self, key: Key) -> Result<(), SMTError> {
        let key = key.to_string().parse::<Node>().unwrap();

        let EntryResponse {
            entry,
            mut siblings,
            ..
        } = self.retrieve_entry(key.clone());

        if entry.get(1).is_none() {
            return Err(SMTError::KeyDoesNotExist(key.to_string()));
        }

        let path = key_to_path(&key.to_string());

        let node = (self.hash)(entry.clone());
        self.nodes.remove(&node);

        self.root = self.zero_node.clone();

        // If there are siblings, the old nodes are deleted and will be re-created below with new hashes.
        if !siblings.is_empty() {
            self.delete_old_nodes(node.clone(), &path, &siblings);

            // If the last sibling is not a leaf node, it adds all the nodes of the path starting from
            // a zero node, otherwise it removes the last non-zero sibling from the `siblings` vector
            // and it starts from it by skipping the last zero nodes.
            if !self.is_leaf(&siblings.last().cloned().unwrap()) {
                self.root = self
                    .add_new_nodes(self.zero_node.clone(), &path, &siblings, None)
                    .unwrap();
            } else {
                let first_sibling = siblings.pop().unwrap();
                let i = get_index_of_last_non_zero_element(
                    siblings
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<String>>()
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>(),
                );

                self.root = self.add_new_nodes(first_sibling, &path, &siblings, Some(i))?;
            }
        }

        Ok(())
    }

    /// Creates a proof to prove the membership or the non-membership of a tree entry.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to create the proof for.
    ///
    /// # Returns
    ///
    /// A `MerkleProof` containing the proof information.
    pub fn create_proof(&self, key: Key) -> MerkleProof {
        let key = key.to_string().parse::<Node>().unwrap();

        let EntryResponse {
            entry,
            matching_entry,
            siblings,
        } = self.retrieve_entry(key);

        // If the key exists, the function returns a proof with the entry itself, otherwise it returns
        // a non-membership proof with the matching entry.
        MerkleProof {
            entry_response: EntryResponse {
                entry: entry.clone(),
                matching_entry,
                siblings,
            },
            root: self.root.clone(),
            membership: entry.get(1).is_some(),
        }
    }

    /// Verifies a membership or a non-membership proof for a given key in the SMT.
    ///
    /// # Arguments
    ///
    /// * `merkle_proof` - The Merkle proof to verify.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the proof is valid or not.
    pub fn verify_proof(&self, merkle_proof: MerkleProof) -> bool {
        // If there is no matching entry, it simply obtains the root hash by using the siblings and the
        // path of the key.
        if merkle_proof.entry_response.matching_entry.is_none() {
            let path = key_to_path(&merkle_proof.entry_response.entry[0].to_string());
            // If there is not an entry value, the proof is a non-membership proof. In this case, since there
            // is not a matching entry, the node is set to a zero node. If there is an entry value, the proof
            // is a membership proof and the node is set to the hash of the entry.
            let node = if merkle_proof.entry_response.entry.get(1).is_some() {
                (self.hash)(merkle_proof.entry_response.entry)
            } else {
                self.zero_node.clone()
            };
            let root = self.calculate_root(node, &path, &merkle_proof.entry_response.siblings);

            // If the obtained root is equal to the proof root, then the proof is valid.
            return root == merkle_proof.root;
        }

        // If there is a matching entry, the proof is definitely a non-membership proof. In this case, it checks
        // if the matching node belongs to the tree, and then it checks if the number of the first matching bits
        // of the keys is greater than or equal to the number of the siblings.
        if let Some(matching_entry) = &merkle_proof.entry_response.matching_entry {
            let matching_path = key_to_path(&matching_entry[0].to_string());
            let node = (self.hash)(matching_entry.to_vec());
            let root =
                self.calculate_root(node, &matching_path, &merkle_proof.entry_response.siblings);

            if merkle_proof.membership == (root == merkle_proof.root) {
                let path = key_to_path(&merkle_proof.entry_response.entry[0].to_string());
                // Returns the first common bits of the two keys: the non-member key and the matching key.
                let first_matching_bits = get_first_common_elements(&path, &matching_path);

                // If the non-member key was a key of a tree entry, the depth of the matching node should be
                // greater than the number of the fisrt matching bits. Otherwise, the depth of the node can be
                // defined by the number of its siblings.
                return merkle_proof.entry_response.siblings.len() <= first_matching_bits.len();
            }
        }

        false
    }

    /// Retrieves the entry associated with the given key from the SMT.
    ///
    /// If the key passed as parameter exists in the SMT, the function returns the entry itself, otherwise
    /// it returns the entry with only the key. When there is another matching entry in the same path, it
    /// returns the matching entry as well.
    ///
    /// In any case, the function returns the siblings of the path.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to retrieve the entry for.
    ///
    /// # Returns
    ///
    /// An `EntryResponse` struct containing the entry, the matching entry (if any), and the siblings of the leaf node.
    fn retrieve_entry(&self, key: Key) -> EntryResponse {
        let path = key_to_path(&key.to_string());
        let mut siblings: Siblings = Vec::new();
        let mut node = self.root.clone();

        let mut i = 0;

        // Starting from the root, it traverses the tree until it reaches a leaf node, a zero node,
        // or a matching entry.
        while node != self.zero_node {
            let child_nodes = self.nodes.get(&node).unwrap_or(&Vec::new()).clone();
            let direction = path[i];

            // If the third element of the child nodes is not None, it means that the node is an entry of the tree.
            if child_nodes.get(2).is_some() {
                if child_nodes[0] == key {
                    // An entry is found with the same key, and it returns it with the siblings.
                    return EntryResponse {
                        entry: child_nodes,
                        matching_entry: None,
                        siblings,
                    };
                }

                // An entry was found with a different key, but the key of this particular entry matches the first 'i'
                // bits of the key passed as parameter. It can be useful in several functions.
                return EntryResponse {
                    entry: vec![key.clone()],
                    matching_entry: Some(child_nodes),
                    siblings,
                };
            }

            // When it goes down into the tree and follows the path, in every step a node is chosen between left
            // and right child nodes, and the opposite node is saved in the `siblings` vector.
            node = child_nodes[direction].clone();
            siblings.push(child_nodes[1 - direction].clone());

            i += 1;
        }

        // The path led to a zero node.
        EntryResponse {
            entry: vec![key],
            matching_entry: None,
            siblings,
        }
    }

    /// Calculates the root of the tree by using the given node, the path, and the siblings.
    ///
    /// It calculates with a bottom up approach by starting from the node and going up to the root.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to start the calculation from.
    /// * `path` - The path of the key.
    /// * `siblings` - The siblings of the path.
    ///
    /// # Returns
    ///
    /// The root of the tree.
    fn calculate_root(&self, mut node: Node, path: &[usize], siblings: &Siblings) -> Node {
        for i in (0..siblings.len()).rev() {
            let child_nodes: ChildNodes = if path[i] != 0 {
                vec![siblings[i].clone(), node.clone()]
            } else {
                vec![node.clone(), siblings[i].clone()]
            };

            node = (self.hash)(child_nodes);
        }

        node
    }

    /// Adds new nodes to the tree with the new hashes.
    ///
    /// It starts with a bottom up approach until it reaches the root of the tree.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to start the calculation from.
    /// * `path` - The path of the key.
    /// * `siblings` - The siblings of the path.
    /// * `i` - The index of the sibling to start from.
    ///
    /// # Returns
    ///
    /// The new root of the tree.
    fn add_new_nodes(
        &mut self,
        mut node: Node,
        path: &[usize],
        siblings: &Siblings,
        i: Option<isize>,
    ) -> Result<Node, SMTError> {
        let mut starting_index = if let Some(i) = i {
            i
        } else {
            siblings.len() as isize - 1
        };

        while starting_index > 0 {
            if siblings.get(starting_index as usize).is_none() {
                return Err(SMTError::InvalidSiblingIndex);
            }

            let child_nodes: ChildNodes = if path[starting_index as usize] == 1 {
                vec![siblings[starting_index as usize].clone(), node.clone()]
            } else {
                vec![node.clone(), siblings[starting_index as usize].clone()]
            };

            node = (self.hash)(child_nodes.clone());

            self.nodes.insert(node.clone(), child_nodes);

            starting_index -= 1;
        }

        Ok(node)
    }

    /// Deletes the old nodes of the tree.
    ///
    /// It starts with a bottom up approach until it reaches the root of the tree.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to start the calculation from.
    /// * `path` - The path of the key.
    /// * `siblings` - The siblings of the path.
    fn delete_old_nodes(&mut self, mut node: Node, path: &[usize], siblings: &Siblings) {
        for i in (0..siblings.len()).rev() {
            let child_nodes: ChildNodes = if path[i] == 1 {
                vec![siblings[i].clone(), node.clone()]
            } else {
                vec![node.clone(), siblings[i].clone()]
            };

            node = (self.hash)(child_nodes);

            self.nodes.remove(&node);
        }
    }

    /// Checks if the given node is a leaf node or not.
    ///
    /// # Arguments
    ///
    /// * `node` - The node to check.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the node is a leaf node or not.
    fn is_leaf(&self, node: &Node) -> bool {
        if let Some(child_nodes) = self.nodes.get(node) {
            child_nodes.get(2).is_some()
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash_function(nodes: Vec<Node>) -> Node {
        let strings: Vec<String> = nodes.iter().map(|node| node.to_string()).collect();
        Node::Str(strings.join(","))
    }

    #[test]
    fn test_new() {
        let smt = SMT::new(hash_function, false);
        assert!(!smt.big_numbers);
        assert_eq!(smt.zero_node, Node::Str("0".to_string()));
        assert_eq!(smt.entry_mark, Node::Str("1".to_string()));
        assert_eq!(smt.nodes, HashMap::new());
        assert_eq!(smt.root, Node::Str("0".to_string()));

        let smt = SMT::new(hash_function, true);
        assert!(smt.big_numbers);
        assert_eq!(smt.zero_node, Node::BigInt(BigInt::from(0)));
        assert_eq!(smt.entry_mark, Node::BigInt(BigInt::from(1)));
        assert_eq!(smt.nodes, HashMap::new());
        assert_eq!(smt.root, Node::BigInt(BigInt::from(0)));
    }

    #[test]
    fn test_get() {
        let mut smt = SMT::new(hash_function, false);
        let key = Key::Str("aaa".to_string());
        let value = Value::Str("bbb".to_string());
        let _ = smt.add(key.clone(), value.clone());
        let result = smt.get(key.clone());
        assert_eq!(result, Some(value));

        let key2 = Key::Str("ccc".to_string());
        let result2 = smt.get(key2.clone());
        assert_eq!(result2, None);

        let mut smt = SMT::new(hash_function, true);
        let key = Key::BigInt(BigInt::from(123));
        let value = Value::BigInt(BigInt::from(456));
        let _ = smt.add(key.clone(), value.clone());
        let result = smt.get(key.clone());
        assert_eq!(result, Some(value));
    }
    #[test]
    fn test_add() {
        let mut smt = SMT::new(hash_function, false);
        let key = Key::Str("aaa".to_string());
        let value = Value::Str("bbb".to_string());
        let result = smt.add(key.clone(), value.clone());
        assert!(result.is_ok());
        assert_eq!(smt.nodes.len(), 1);
        assert_eq!(
            smt.nodes.get(&smt.root),
            Some(&vec![key.clone(), value.clone(), smt.entry_mark.clone()])
        );

        let mut smt = SMT::new(hash_function, true);
        let key = Key::BigInt(BigInt::from(123));
        let value = Value::BigInt(BigInt::from(456));
        let result = smt.add(key.clone(), value.clone());
        assert!(result.is_ok());
        assert_eq!(smt.nodes.len(), 1);
        assert_eq!(
            smt.nodes.get(&smt.root),
            Some(&vec![key.clone(), value.clone(), smt.entry_mark.clone()])
        );
    }

    #[test]
    fn test_update() {
        let mut smt = SMT::new(hash_function, false);
        let key = Key::Str("aaa".to_string());
        let value = Value::Str("bbb".to_string());
        let _ = smt.add(key.clone(), value.clone());

        let new_value = Value::Str("ccc".to_string());
        let result = smt.update(key.clone(), new_value.clone());
        assert!(result.is_ok());
        assert_eq!(smt.nodes.len(), 1);
        assert_eq!(
            smt.nodes.get(&smt.root),
            Some(&vec![
                key.clone(),
                new_value.clone(),
                smt.entry_mark.clone()
            ])
        );

        let key2 = Key::Str("def".to_string());
        let result2 = smt.update(key2.clone(), new_value.clone());
        assert_eq!(result2, Err(SMTError::KeyDoesNotExist(key2.to_string())));

        let mut smt = SMT::new(hash_function, true);
        let key = Key::BigInt(BigInt::from(123));
        let value = Value::BigInt(BigInt::from(456));
        let _ = smt.add(key.clone(), value.clone());

        let new_value = Value::BigInt(BigInt::from(789));
        let result = smt.update(key.clone(), new_value.clone());
        assert!(result.is_ok());
        assert_eq!(smt.nodes.len(), 1);
        assert_eq!(
            smt.nodes.get(&smt.root),
            Some(&vec![
                key.clone(),
                new_value.clone(),
                smt.entry_mark.clone()
            ])
        );
    }

    #[test]
    fn test_delete() {
        let mut smt = SMT::new(hash_function, false);
        let key = Key::Str("abc".to_string());
        let value = Value::Str("123".to_string());
        let _ = smt.add(key.clone(), value.clone());
        let result = smt.delete(key.clone());
        assert!(result.is_ok());
        assert_eq!(smt.nodes.len(), 0);
        assert_eq!(smt.root, smt.zero_node);

        let key2 = Key::Str("def".to_string());
        let result2 = smt.delete(key2.clone());
        assert_eq!(result2, Err(SMTError::KeyDoesNotExist(key2.to_string())));

        let mut smt = SMT::new(hash_function, true);
        let key = Key::BigInt(BigInt::from(123));
        let value = Value::BigInt(BigInt::from(456));
        let _ = smt.add(key.clone(), value.clone());
        let result = smt.delete(key.clone());
        assert!(result.is_ok());
        assert_eq!(smt.nodes.len(), 0);
        assert_eq!(smt.root, smt.zero_node);
    }

    #[test]
    fn test_create_proof() {
        let mut smt = SMT::new(hash_function, false);
        let key = Key::Str("abc".to_string());
        let value = Value::Str("123".to_string());
        let _ = smt.add(key.clone(), value.clone());
        let proof = smt.create_proof(key.clone());
        assert_eq!(proof.root, smt.root);

        let mut smt = SMT::new(hash_function, true);
        let key = Key::BigInt(BigInt::from(123));
        let value = Value::BigInt(BigInt::from(456));
        let _ = smt.add(key.clone(), value.clone());
        let proof = smt.create_proof(key.clone());
        assert_eq!(proof.root, smt.root);
    }

    #[test]
    fn test_verify_proof() {
        let mut smt = SMT::new(hash_function, false);
        let key = Key::Str("abc".to_string());
        let value = Value::Str("123".to_string());
        let _ = smt.add(key.clone(), value.clone());
        let proof = smt.create_proof(key.clone());
        let result = smt.verify_proof(proof);
        assert!(result);

        let key2 = Key::Str("def".to_string());
        let false_proof = MerkleProof {
            entry_response: EntryResponse {
                entry: vec![key2.clone()],
                matching_entry: None,
                siblings: Vec::new(),
            },
            root: smt.root.clone(),
            membership: false,
        };
        let fun = smt.verify_proof(false_proof);
        assert!(!fun);

        let mut smt = SMT::new(hash_function, true);
        let key = Key::BigInt(BigInt::from(123));
        let value = Value::BigInt(BigInt::from(456));
        let _ = smt.add(key.clone(), value.clone());
        let proof = smt.create_proof(key.clone());
        let result = smt.verify_proof(proof);
        assert!(result);

        let key2 = Key::BigInt(BigInt::from(789));
        let false_proof = MerkleProof {
            entry_response: EntryResponse {
                entry: vec![key2.clone()],
                matching_entry: None,
                siblings: Vec::new(),
            },
            root: smt.root.clone(),
            membership: true,
        };
        let fun = smt.verify_proof(false_proof);
        assert!(!fun);
    }

    #[test]
    fn test_retrieve_entry() {
        let smt = SMT::new(hash_function, false);
        let key = Key::Str("be12".to_string());
        let entry_response = smt.retrieve_entry(key.clone());
        assert_eq!(entry_response.entry, vec![key]);
        assert_eq!(entry_response.matching_entry, None);
        assert_eq!(entry_response.siblings, Vec::new());

        let smt = SMT::new(hash_function, true);
        let key = Key::BigInt(BigInt::from(123));
        let entry_response = smt.retrieve_entry(key.clone());
        assert_eq!(entry_response.entry, vec![key]);
        assert_eq!(entry_response.matching_entry, None);
        assert_eq!(entry_response.siblings, Vec::new());
    }

    #[test]
    fn test_calculate_root() {
        let smt = SMT::new(hash_function, false);
        let node = Node::Str("node".to_string());
        let path = &[0, 1, 0];
        let siblings = vec![
            Node::Str("sibling1".to_string()),
            Node::Str("sibling2".to_string()),
            Node::Str("sibling3".to_string()),
        ];
        let root = smt.calculate_root(node.clone(), path, &siblings);
        assert_eq!(
            root,
            Node::Str("sibling2,node,sibling3,sibling1".to_string())
        );

        let smt = SMT::new(hash_function, true);
        let node = Node::BigInt(BigInt::from(123));
        let path = &[1, 0];
        let siblings = vec![
            Node::BigInt(BigInt::from(456)),
            Node::BigInt(BigInt::from(789)),
        ];
        let root = smt.calculate_root(node.clone(), path, &siblings);
        assert_eq!(root, Node::Str("456,123,789".to_string()));
    }

    #[test]
    fn test_add_new_nodes() {
        let mut smt = SMT::new(hash_function, false);
        let node = Node::Str("node".to_string());
        let path = &[0, 1, 0];
        let siblings = vec![
            Node::Str("sibling1".to_string()),
            Node::Str("sibling2".to_string()),
            Node::Str("sibling3".to_string()),
        ];
        let new_node = smt
            .add_new_nodes(node.clone(), path, &siblings, None)
            .unwrap();
        assert_eq!(new_node, Node::Str("sibling2,sibling3,node".to_string()));

        let starting_index = smt
            .add_new_nodes(node.clone(), path, &siblings, Some(1))
            .unwrap();
        assert_eq!(starting_index, Node::Str("sibling2,node".to_string()));

        let mut smt = SMT::new(hash_function, true);
        let node = Node::BigInt(BigInt::from(111));
        let path = &[1, 0, 0];
        let siblings = vec![
            Node::BigInt(BigInt::from(222)),
            Node::BigInt(BigInt::from(333)),
            Node::BigInt(BigInt::from(444)),
        ];
        let new_node = smt
            .add_new_nodes(node.clone(), path, &siblings, None)
            .unwrap();
        assert_eq!(new_node, Node::Str("333,444,111".to_string()));

        let starting_index = smt
            .add_new_nodes(node.clone(), path, &siblings, Some(1))
            .unwrap();
        assert_eq!(starting_index, Node::Str("333,111".to_string()));
    }

    #[test]
    fn test_delete_old_nodes() {
        let mut smt = SMT::new(hash_function, false);
        let node = Node::Str("abc".to_string());
        let path = &[0, 1, 0];
        let siblings = vec![
            Node::Str("sibling1".to_string()),
            Node::Str("sibling2".to_string()),
            Node::Str("sibling3".to_string()),
        ];
        let new_node = smt
            .add_new_nodes(node.clone(), path, &siblings, None)
            .unwrap();
        assert_eq!(new_node, Node::Str("sibling2,sibling3,abc".to_string()));
        smt.delete_old_nodes(node.clone(), path, &siblings);
        assert_eq!(smt.nodes.len(), 0);

        let mut smt = SMT::new(hash_function, true);
        let node = Node::BigInt(BigInt::from(123));
        let path = &[1, 0];
        let siblings = vec![
            Node::BigInt(BigInt::from(456)),
            Node::BigInt(BigInt::from(789)),
        ];
        let new_node = smt
            .add_new_nodes(node.clone(), path, &siblings, None)
            .unwrap();
        assert_eq!(new_node, Node::Str("789,123".to_string()));
        smt.delete_old_nodes(node.clone(), path, &siblings);
        assert_eq!(smt.nodes.len(), 0);
    }

    #[test]
    fn test_is_leaf() {
        let mut smt = SMT::new(hash_function, false);
        let node = Node::Str("abc".to_string());
        assert!(!smt.is_leaf(&node));

        smt.nodes.insert(
            Node::Str("abc".to_string()),
            vec![
                Node::Str("123".to_string()),
                Node::Str("456".to_string()),
                Node::Str("789".to_string()),
            ],
        );
        assert!(smt.is_leaf(&node));

        let mut smt = SMT::new(hash_function, true);
        let node = Node::BigInt(BigInt::from(123));
        assert!(!smt.is_leaf(&node));

        smt.nodes.insert(
            Node::BigInt(BigInt::from(123)),
            vec![
                Node::BigInt(BigInt::from(111)),
                Node::BigInt(BigInt::from(222)),
                Node::BigInt(BigInt::from(333)),
            ],
        );
        assert!(smt.is_leaf(&node));
    }
}
