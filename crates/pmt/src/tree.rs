use crate::*;

use std::cmp::{max, min};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// db[DEPTH_KEY] = depth
const DEPTH_KEY: DBKey = (u64::MAX - 1).to_be_bytes();

// db[NEXT_INDEX_KEY] = next_index;
const NEXT_INDEX_KEY: DBKey = u64::MAX.to_be_bytes();

// Default tree depth
const DEFAULT_TREE_DEPTH: usize = 20;

// Denotes keys (depth, index) in Merkle Tree. Can be converted to DBKey
// TODO! Think about using hashing for that
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key(usize, usize);
impl From<Key> for DBKey {
    fn from(key: Key) -> Self {
        let cantor_pairing = ((key.0 + key.1) * (key.0 + key.1 + 1) / 2 + key.1) as u64;
        cantor_pairing.to_be_bytes()
    }
}

/// The Merkle Tree structure
pub struct MerkleTree<D, H>
where
    D: Database,
    H: Hasher,
{
    pub db: D,
    depth: usize,
    next_index: usize,
    cache: Vec<H::Fr>,
    root: H::Fr,
}

/// The Merkle proof structure
#[derive(Clone, PartialEq, Eq)]
pub struct MerkleProof<H: Hasher>(pub Vec<(H::Fr, u8)>);

impl<D, H> MerkleTree<D, H>
where
    D: Database,
    H: Hasher,
{
    /// Creates tree with specified depth and default "pmtree_db" dbpath.
    pub fn default(depth: usize) -> PmtreeResult<Self> {
        Self::new(depth, D::Config::default())
    }

    /// Creates new `MerkleTree` and store it to the specified path/db
    pub fn new(depth: usize, db_config: D::Config) -> PmtreeResult<Self> {
        // Create new db instance
        let mut db = D::new(db_config)?;

        // Insert depth val into db
        let depth_val = depth.to_be_bytes().to_vec();
        db.put(DEPTH_KEY, depth_val)?;

        // Insert next_index val into db
        let next_index = 0usize;
        let next_index_val = next_index.to_be_bytes().to_vec();
        db.put(NEXT_INDEX_KEY, next_index_val)?;

        // Cache nodes
        let mut cache = vec![H::default_leaf(); depth + 1];

        // Initialize one branch of the `Merkle Tree` from bottom to top
        cache[depth] = H::default_leaf();
        db.put(Key(depth, 0).into(), H::serialize(cache[depth]))?;
        for i in (0..depth).rev() {
            cache[i] = H::hash(&[cache[i + 1], cache[i + 1]]);
            db.put(Key(i, 0).into(), H::serialize(cache[i]))?;
        }

        let root = cache[0];

        Ok(Self {
            db,
            depth,
            next_index,
            cache,
            root,
        })
    }

    /// Loads existing Merkle Tree from the specified path/db
    pub fn load(db_config: D::Config) -> PmtreeResult<Self> {
        // Load existing db instance
        let db = D::load(db_config)?;

        // Load root
        let root = match db.get(Key(0, 0).into())? {
            Some(root) => H::deserialize(root),
            None => H::default_leaf(),
        };

        // Load depth & next_index values from db
        let depth = match db.get(DEPTH_KEY)? {
            Some(depth) => usize::from_be_bytes(depth.try_into().unwrap()),
            None => DEFAULT_TREE_DEPTH,
        };

        let next_index = match db.get(NEXT_INDEX_KEY)? {
            Some(next_index) => usize::from_be_bytes(next_index.try_into().unwrap()),
            None => 0,
        };

        // Load cache vec
        let mut cache = vec![H::default_leaf(); depth + 1];
        cache[depth] = H::default_leaf();
        for i in (0..depth).rev() {
            cache[i] = H::hash(&[cache[i + 1], cache[i + 1]]);
        }

        Ok(Self {
            db,
            depth,
            next_index,
            cache,
            root,
        })
    }

    /// Closes the db connection
    pub fn close(&mut self) -> PmtreeResult<()> {
        self.db.close()
    }

    /// Sets a leaf at the specified tree index
    pub fn set(&mut self, key: usize, leaf: H::Fr) -> PmtreeResult<()> {
        if key >= self.capacity() {
            return Err(PmtreeErrorKind::TreeError(TreeErrorKind::IndexOutOfBounds));
        }

        self.db
            .put(Key(self.depth, key).into(), H::serialize(leaf))?;
        self.recalculate_from(key)?;

        // Update next_index in memory
        self.next_index = max(self.next_index, key + 1);

        // Update next_index in db
        let next_index_val = self.next_index.to_be_bytes().to_vec();
        self.db.put(NEXT_INDEX_KEY, next_index_val)?;

        Ok(())
    }

    // Recalculates `Merkle Tree` from the specified key
    fn recalculate_from(&mut self, key: usize) -> PmtreeResult<()> {
        let mut depth = self.depth;
        let mut i = key;

        loop {
            let value = self.hash_couple(depth, i)?;
            i >>= 1;
            depth -= 1;
            self.db.put(Key(depth, i).into(), H::serialize(value))?;

            if depth == 0 {
                self.root = value;
                break;
            }
        }

        Ok(())
    }

    // Hashes the correct couple for the key
    fn hash_couple(&self, depth: usize, key: usize) -> PmtreeResult<H::Fr> {
        let b = key & !1;
        Ok(H::hash(&[
            self.get_elem(Key(depth, b))?,
            self.get_elem(Key(depth, b + 1))?,
        ]))
    }

    // Returns elem by the key
    pub fn get_elem(&self, key: Key) -> PmtreeResult<H::Fr> {
        let res = self
            .db
            .get(key.into())?
            .map_or(self.cache[key.0], |value| H::deserialize(value));

        Ok(res)
    }

    /// Deletes a leaf at the `key` by setting it to its default value
    pub fn delete(&mut self, key: usize) -> PmtreeResult<()> {
        if key >= self.next_index {
            return Err(PmtreeErrorKind::TreeError(TreeErrorKind::InvalidKey));
        }

        self.set(key, H::default_leaf())?;

        Ok(())
    }

    /// Inserts a leaf to the next available index
    pub fn update_next(&mut self, leaf: H::Fr) -> PmtreeResult<()> {
        self.set(self.next_index, leaf)?;

        Ok(())
    }

    /// Batch insertion from starting index
    pub fn set_range<I: IntoIterator<Item = H::Fr>>(
        &mut self,
        start: usize,
        leaves: I,
    ) -> PmtreeResult<()> {
        self.batch_insert(
            Some(start),
            leaves.into_iter().collect::<Vec<_>>().as_slice(),
        )
    }

    /// Batch insertion, updates the tree in parallel.
    pub fn batch_insert(&mut self, start: Option<usize>, leaves: &[H::Fr]) -> PmtreeResult<()> {
        let start = start.unwrap_or(self.next_index);
        let end = start + leaves.len();

        if end > self.capacity() {
            return Err(PmtreeErrorKind::TreeError(TreeErrorKind::MerkleTreeIsFull));
        }

        let mut subtree = HashMap::<Key, H::Fr>::new();

        let root_key = Key(0, 0);

        subtree.insert(root_key, self.root);
        self.fill_nodes(root_key, start, end, &mut subtree, leaves, start)?;

        let subtree = Arc::new(RwLock::new(subtree));

        let root_val = rayon::ThreadPoolBuilder::new()
            .num_threads(rayon::current_num_threads())
            .build()
            .unwrap()
            .install(|| Self::batch_recalculate(root_key, Arc::clone(&subtree), self.depth));

        let subtree = RwLock::into_inner(Arc::try_unwrap(subtree).unwrap()).unwrap();

        self.db.put_batch(
            subtree
                .into_iter()
                .map(|(key, value)| (key.into(), H::serialize(value)))
                .collect(),
        )?;

        // Update next_index value in db
        if end > self.next_index {
            self.next_index = end;
            self.db
                .put(NEXT_INDEX_KEY, self.next_index.to_be_bytes().to_vec())?;
        }

        // Update root value in memory
        self.root = root_val;

        Ok(())
    }

    // Fills hashmap subtree
    fn fill_nodes(
        &self,
        key: Key,
        start: usize,
        end: usize,
        subtree: &mut HashMap<Key, H::Fr>,
        leaves: &[H::Fr],
        from: usize,
    ) -> PmtreeResult<()> {
        if key.0 == self.depth {
            if key.1 >= from {
                subtree.insert(key, leaves[key.1 - from]);
            }
            return Ok(());
        }

        let left = Key(key.0 + 1, key.1 * 2);
        let right = Key(key.0 + 1, key.1 * 2 + 1);

        let left_val = self.get_elem(left)?;
        let right_val = self.get_elem(right)?;

        subtree.insert(left, left_val);
        subtree.insert(right, right_val);

        let half = 1 << (self.depth - key.0 - 1);

        if start < half {
            self.fill_nodes(left, start, min(end, half), subtree, leaves, from)?;
        }

        if end > half {
            self.fill_nodes(right, 0, end - half, subtree, leaves, from)?;
        }

        Ok(())
    }

    // Recalculates tree in parallel (in-memory)
    fn batch_recalculate(
        key: Key,
        subtree: Arc<RwLock<HashMap<Key, H::Fr>>>,
        depth: usize,
    ) -> H::Fr {
        let left_child = Key(key.0 + 1, key.1 * 2);
        let right_child = Key(key.0 + 1, key.1 * 2 + 1);

        if key.0 == depth || !subtree.read().unwrap().contains_key(&left_child) {
            return *subtree.read().unwrap().get(&key).unwrap();
        }

        let (left, right) = rayon::join(
            || Self::batch_recalculate(left_child, Arc::clone(&subtree), depth),
            || Self::batch_recalculate(right_child, Arc::clone(&subtree), depth),
        );

        let result = H::hash(&[left, right]);

        subtree.write().unwrap().insert(key, result);

        result
    }

    /// Computes a Merkle proof for the leaf at the specified index
    pub fn proof(&self, index: usize) -> PmtreeResult<MerkleProof<H>> {
        if index >= self.capacity() {
            return Err(PmtreeErrorKind::TreeError(TreeErrorKind::IndexOutOfBounds));
        }

        let mut witness = Vec::with_capacity(self.depth);

        let mut i = index;
        let mut depth = self.depth;
        while depth != 0 {
            i ^= 1;
            witness.push((
                self.get_elem(Key(depth, i))?,
                (1 - (i & 1)).try_into().unwrap(),
            ));
            i >>= 1;
            depth -= 1;
        }

        Ok(MerkleProof(witness))
    }

    /// Verifies a Merkle proof with respect to the input leaf and the tree root
    pub fn verify(&self, leaf: &H::Fr, witness: &MerkleProof<H>) -> bool {
        let expected_root = witness.compute_root_from(leaf);

        self.root() == expected_root
    }

    /// Returns the leaf by the key
    pub fn get(&self, key: usize) -> PmtreeResult<H::Fr> {
        if key >= self.capacity() {
            return Err(PmtreeErrorKind::TreeError(TreeErrorKind::IndexOutOfBounds));
        }

        self.get_elem(Key(self.depth, key))
    }

    /// Returns the root of the tree
    pub fn root(&self) -> H::Fr {
        self.root
    }

    /// Returns the total number of leaves set
    pub fn leaves_set(&self) -> usize {
        self.next_index
    }

    /// Returns the capacity of the tree, i.e. the maximum number of leaves
    pub fn capacity(&self) -> usize {
        1 << self.depth
    }

    /// Returns the depth of the tree
    pub fn depth(&self) -> usize {
        self.depth
    }
}

impl<H: Hasher> MerkleProof<H> {
    /// Computes the Merkle root by iteratively hashing specified Merkle proof with specified leaf
    pub fn compute_root_from(&self, leaf: &H::Fr) -> H::Fr {
        let mut acc = *leaf;
        for w in self.0.iter() {
            if w.1 == 0 {
                acc = H::hash(&[acc, w.0]);
            } else {
                acc = H::hash(&[w.0, acc]);
            }
        }

        acc
    }

    /// Computes the leaf index corresponding to a Merkle proof
    pub fn leaf_index(&self) -> usize {
        self.get_path_index()
            .into_iter()
            .rev()
            .fold(0, |acc, digit| (acc << 1) + usize::from(digit))
    }

    /// Returns the path indexes forming a Merkle Proof
    pub fn get_path_index(&self) -> Vec<u8> {
        self.0.iter().map(|x| x.1).collect()
    }

    /// Returns the path elements forming a Merkle proof
    pub fn get_path_elements(&self) -> Vec<H::Fr> {
        self.0.iter().map(|x| x.0).collect()
    }

    /// Returns the length of a Merkle proof
    pub fn length(&self) -> usize {
        self.0.len()
    }
}
