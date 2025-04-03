use std::collections::HashSet;
pub struct IMT {
    nodes: Vec<Vec<IMTNode>>,
    zeroes: Vec<IMTNode>,
    hash: IMTHashFunction,
    depth: usize,
    arity: usize,
}

pub struct IMTMerkleProof {
    root: IMTNode,
    leaf: IMTNode,
    path_indices: Vec<usize>,
    siblings: Vec<Vec<IMTNode>>,
}

pub type IMTNode = String;
pub type IMTHashFunction = fn(Vec<IMTNode>) -> IMTNode;

impl IMT {
    pub fn new(
        hash: IMTHashFunction,
        depth: usize,
        zero_value: IMTNode,
        arity: usize,
        leaves: Vec<IMTNode>,
    ) -> Result<IMT, &'static str> {
        if leaves.len() > arity.pow(depth as u32) {
            return Err("The tree cannot contain more than arity^depth leaves");
        }

        let mut imt = IMT {
            nodes: vec![vec![]; depth + 1],
            zeroes: vec![],
            hash,
            depth,
            arity,
        };

        let mut current_zero = zero_value;
        for _ in 0..depth {
            imt.zeroes.push(current_zero.clone());
            current_zero = (imt.hash)(vec![current_zero; arity]);
        }

        imt.nodes[0] = leaves;

        for level in 0..depth {
            for index in 0..((imt.nodes[level].len() as f64 / arity as f64).ceil() as usize) {
                let position = index * arity;
                let children: Vec<_> = (0..arity)
                    .map(|i| {
                        imt.nodes[level]
                            .get(position + i)
                            .cloned()
                            .unwrap_or_else(|| imt.zeroes[level].clone())
                    })
                    .collect();

                if let Some(next_level) = imt.nodes.get_mut(level + 1) {
                    next_level.push((imt.hash)(children));
                }
            }
        }

        Ok(imt)
    }

    pub fn root(&mut self) -> Option<IMTNode> {
        self.nodes[self.depth].first().cloned()
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn nodes(&self) -> Vec<Vec<IMTNode>> {
        self.nodes.clone()
    }

    pub fn zeroes(&self) -> Vec<IMTNode> {
        self.zeroes.clone()
    }

    pub fn leaves(&self) -> Vec<IMTNode> {
        self.nodes[0].clone()
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn insert(&mut self, leaf: IMTNode) -> Result<(), &'static str> {
        if self.nodes[0].len() >= self.arity.pow(self.depth as u32) {
            return Err("The tree is full");
        }

        let index = self.nodes[0].len();
        self.nodes[0].push(leaf);
        self.update(index, self.nodes[0][index].clone())
    }

    pub fn update(&mut self, mut index: usize, new_leaf: IMTNode) -> Result<(), &'static str> {
        if index >= self.nodes[0].len() {
            return Err("The leaf does not exist in this tree");
        }

        let mut node = new_leaf;
        self.nodes[0][index].clone_from(&node);

        for level in 0..self.depth {
            let position = index % self.arity;
            let level_start_index = index - position;
            let level_end_index = level_start_index + self.arity;

            let children: Vec<_> = (level_start_index..level_end_index)
                .map(|i| {
                    self.nodes[level]
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| self.zeroes[level].clone())
                })
                .collect();

            node = (self.hash)(children);
            index /= self.arity;

            if self.nodes[level + 1].len() <= index {
                self.nodes[level + 1].push(node.clone());
            } else {
                self.nodes[level + 1][index].clone_from(&node);
            }
        }

        Ok(())
    }

    pub fn delete(&mut self, index: usize) -> Result<(), &'static str> {
        self.update(index, self.zeroes[0].clone())
    }

    pub fn batch_delete(&mut self, indices: Vec<usize>) -> Result<(), &'static str> {
        let updates: Vec<(usize, IMTNode)> = indices.into_iter().map(|index| (index, self.zeroes[0].clone())).collect();
        self.batch_update(updates)
    }

    pub fn create_proof(&self, index: usize) -> Result<IMTMerkleProof, &'static str> {
        if index >= self.nodes[0].len() {
            return Err("The leaf does not exist in this tree");
        }

        let mut siblings = Vec::with_capacity(self.depth);
        let mut path_indices = Vec::with_capacity(self.depth);
        let mut current_index = index;

        for level in 0..self.depth {
            let position = current_index % self.arity;
            let level_start_index = current_index - position;
            let level_end_index = level_start_index + self.arity;

            path_indices.push(position);
            let mut level_siblings = Vec::new();

            for i in level_start_index..level_end_index {
                if i != current_index {
                    level_siblings.push(
                        self.nodes[level]
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| self.zeroes[level].clone()),
                    );
                }
            }

            siblings.push(level_siblings);
            current_index /= self.arity;
        }

        Ok(IMTMerkleProof {
            root: self.nodes[self.depth][0].clone(),
            leaf: self.nodes[0][index].clone(),
            path_indices,
            siblings,
        })
    }

    pub fn verify_proof(&self, proof: &IMTMerkleProof) -> bool {
        let mut node = proof.leaf.clone();

        for (i, sibling) in proof.siblings.iter().enumerate() {
            let mut children = sibling.clone();
            children.insert(proof.path_indices[i], node);

            node = (self.hash)(children);
        }

        node == proof.root
    }

    pub fn batch_insert(&mut self, leaves: Vec<IMTNode>) -> Result<(), &'static str> {
        let old_len = self.nodes[0].len();
        if old_len + leaves.len() > self.arity.pow(self.depth as u32) {
            return Err("The tree cannot contain more than arity^depth leaves");
        }

        // Append all new leaves to the leaf level
        self.nodes[0].extend(leaves);
        
        // Update the tree level by level, starting from the first new leaf
        let mut start_idx = old_len;
        for level in 0..self.depth {
            let parent_start_idx = start_idx / self.arity;
            let parent_end_idx = ((self.nodes[level].len() as f64) / (self.arity as f64)).ceil() as usize;
            for idx in parent_start_idx..parent_end_idx {
                let position = idx * self.arity;
                let children: Vec<_> = (0..self.arity)
                    .map(|i| {
                        self.nodes[level]
                            .get(position + i)
                            .cloned()
                            .unwrap_or_else(|| self.zeroes[level].clone())
                    })
                    .collect();
                let node = (self.hash)(children);
                if idx < self.nodes[level + 1].len() {
                    self.nodes[level + 1][idx] = node;
                } else {
                    self.nodes[level + 1].push(node);
                }
            }
            start_idx = parent_start_idx;
        }

        Ok(())
    }

    /// Updates multiple leaves at specified indices with new values.
    pub fn batch_update(&mut self, updates: Vec<(usize, IMTNode)>) -> Result<(), &'static str> {
        // Collect indices and validate them in one pass
        let mut updated_indices = HashSet::new();
        for &(index, _) in &updates {
            if index >= self.nodes[0].len() {
                return Err("Index out of range");
            }
            updated_indices.insert(index);
        }
    
        // Apply updates to the leaf level
        for (index, new_value) in updates {
            self.nodes[0][index] = new_value;
        }
    
        // Update the tree level by level using the set of affected indices
        for level in 0..self.depth {
            let mut parent_updated_indices: HashSet<usize> = HashSet::new();
            for &idx in &updated_indices {
                let parent_idx = idx / self.arity;
                if !parent_updated_indices.contains(&parent_idx) {
                    let position = parent_idx * self.arity;
                    let children: Vec<_> = (0..self.arity)
                        .map(|i| {
                            self.nodes[level]
                                .get(position + i)
                                .cloned()
                                .unwrap_or_else(|| self.zeroes[level].clone())
                        })
                        .collect();
                    let node = (self.hash)(children);
                    if parent_idx < self.nodes[level + 1].len() {
                        self.nodes[level + 1][parent_idx] = node;
                    } else {
                        self.nodes[level + 1].push(node);
                    }
                    parent_updated_indices.insert(parent_idx);
                }
            }
            updated_indices = parent_updated_indices;
        }
    
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_hash_function(nodes: Vec<String>) -> String {
        nodes.join(",")
    }

    #[test]
    fn test_new_imt() {
        let hash: IMTHashFunction = simple_hash_function;
        let imt = IMT::new(hash, 3, "zero".to_string(), 2, vec![]);

        assert!(imt.is_ok());
    }

    #[test]
    fn test_insertion() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(hash, 3, "zero".to_string(), 2, vec![]).unwrap();

        assert!(imt.insert("leaf1".to_string()).is_ok());
    }

    #[test] 
    fn test_batch_insert() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(hash, 3, "zero".to_string(), 2, vec![]).unwrap();

        // Insert multiple leaves at once
        let leaves = vec!["leaf1".to_string(), "leaf2".to_string(), "leaf3".to_string()];
        let result = imt.batch_insert(leaves);
        assert!(result.is_ok());

        // Verify the leaves are correctly inserted
        assert_eq!(
            imt.leaves(),
            vec!["leaf1".to_string(), "leaf2".to_string(), "leaf3".to_string()]
        );

        // Check that the root is computed (specific value depends on hash function)
        assert!(imt.root().is_some());
    }

    #[test]
    fn test_delete() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(hash, 3, "zero".to_string(), 2, vec!["leaf1".to_string()]).unwrap();

        assert!(imt.delete(0).is_ok());
    }
    
    #[test]
    fn test_batch_delete() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(
            hash,
            3,
            "zero".to_string(),
            2,
            vec!["leaf1".to_string(), "leaf2".to_string(), "leaf3".to_string()],
        )
        .unwrap();

        // Delete multiple leaves
        let indices = vec![0, 2];
        let result = imt.batch_delete(indices);
        assert!(result.is_ok());

        // Verify deleted leaves are set to zero
        assert_eq!(
            imt.leaves(),
            vec!["zero".to_string(), "leaf2".to_string(), "zero".to_string()]
        );

        // Check that the root is updated
        assert!(imt.root().is_some());
    }
   
    #[test]
    fn test_update() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(hash, 3, "zero".to_string(), 2, vec!["leaf1".to_string()]).unwrap();

        assert!(imt.update(0, "new_leaf".to_string()).is_ok());
    }

    #[test]
    fn test_batch_update() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(
            hash,
            3,
            "zero".to_string(),
            2,
            vec!["leaf1".to_string(), "leaf2".to_string(), "leaf3".to_string()],
        )
        .unwrap();

        // Update multiple leaves
        let updates = vec![
            (0, "new_leaf1".to_string()),
            (2, "new_leaf3".to_string()),
        ];
        let result = imt.batch_update(updates);
        assert!(result.is_ok());

        // Verify the updated leaves
        assert_eq!(
            imt.leaves(),
            vec!["new_leaf1".to_string(), "leaf2".to_string(), "new_leaf3".to_string()]
        );

        // Check that the root is updated
        assert!(imt.root().is_some());
    }
    #[test]
    fn test_create_and_verify_proof() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(hash, 3, "zero".to_string(), 2, vec!["leaf1".to_string()]).unwrap();
        imt.insert("leaf2".to_string()).unwrap();

        let proof = imt.create_proof(0);
        assert!(proof.is_ok());

        let proof = proof.unwrap();
        assert!(imt.verify_proof(&proof));
    }

    #[test]
    fn should_not_initialize_with_too_many_leaves() {
        let hash: IMTHashFunction = simple_hash_function;
        let leaves = vec![
            "leaf1".to_string(),
            "leaf2".to_string(),
            "leaf3".to_string(),
            "leaf4".to_string(),
            "leaf5".to_string(),
        ];
        let imt = IMT::new(hash, 2, "zero".to_string(), 2, leaves);
        assert!(imt.is_err());
    }

    #[test]
    fn should_not_insert_in_full_tree() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(
            hash,
            1,
            "zero".to_string(),
            2,
            vec!["leaf1".to_string(), "leaf2".to_string()],
        )
        .unwrap();

        let result = imt.insert("leaf3".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_many_invalid_index() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(
            hash,
            3,
            "zero".to_string(),
            2,
            vec!["leaf1".to_string(), "leaf2".to_string()],
        )
        .unwrap();

        // Attempt to delete an invalid index
        let indices = vec![0, 2];
        let result = imt.batch_delete(indices);
        assert!(result.is_err());
    }

    #[test]
    fn should_not_delete_nonexistent_leaf() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(hash, 3, "zero".to_string(), 2, vec!["leaf1".to_string()]).unwrap();

        let result = imt.delete(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_root() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(
            hash,
            2,
            "zero".to_string(),
            2,
            vec!["leaf1".to_string(), "leaf2".to_string()],
        )
        .unwrap();

        assert_eq!(imt.root(), Some("leaf1,leaf2,zero,zero".to_string()));
    }

    #[test]
    fn test_leaves() {
        let hash: IMTHashFunction = simple_hash_function;
        let imt = IMT::new(
            hash,
            2,
            "zero".to_string(),
            2,
            vec!["leaf1".to_string(), "leaf2".to_string()],
        )
        .unwrap();

        assert_eq!(imt.leaves(), vec!["leaf1".to_string(), "leaf2".to_string()]);
    }

    #[test]
    fn test_depth_and_arity() {
        let hash: IMTHashFunction = simple_hash_function;
        let imt = IMT::new(hash, 3, "zero".to_string(), 2, vec![]).unwrap();

        assert_eq!(imt.depth(), 3);
        assert_eq!(imt.arity(), 2);
    }

    #[test]
    fn test_batched_operations_with_proof() {
        let hash: IMTHashFunction = simple_hash_function;
        let mut imt = IMT::new(hash, 3, "zero".to_string(), 2, vec![]).unwrap();

        // Perform batched insertions
        let leaves = vec!["leaf1".to_string(), "leaf2".to_string(), "leaf3".to_string()];
        imt.batch_insert(leaves).unwrap();

        // Perform batched updates
        let updates = vec![(1, "new_leaf2".to_string())];
        imt.batch_update(updates).unwrap();

        // Perform batched deletions
        let indices = vec![2];
        imt.batch_delete(indices).unwrap();

        // Create and verify a proof for an existing leaf
        let proof = imt.create_proof(0).unwrap();
        assert!(imt.verify_proof(&proof));
    }

}
