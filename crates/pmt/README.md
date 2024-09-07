<p align="center">
    <h1 align="center">
        Sparse Merkle tree
    </h1>
    <p align="center">Sparse Merkle tree implementation in Rust.</p>
</p>

<div align="center">
    <h4>
        <a href="https://appliedzkp.org/discord">
            🗣️ Chat &amp; Support
        </a>
    </h4>
</div>

Generic for storage Merkle Tree (sparse & fixed-size) in Rust.

## References

- https://github.com/Rate-Limiting-Nullifier/pmtree

---

## 🛠 Install

```bash
cargo add zk-kit-pmt
```

## 📜 Usage

```rust
struct MemoryDB(HashMap<DBKey, Value>);
struct MyKeccak(());

#[derive(Default)]
struct MemoryDBConfig;

impl Database for MemoryDB {
    type Config = MemoryDBConfig;

    fn new(_db_config: MemoryDBConfig) -> PmtreeResult<Self> {
        Ok(MemoryDB(HashMap::new()))
    }

    fn load(_db_config: MemoryDBConfig) -> PmtreeResult<Self> {
        Err(PmtreeErrorKind::DatabaseError(
            DatabaseErrorKind::CannotLoadDatabase,
        ))
    }

    fn get(&self, key: DBKey) -> PmtreeResult<Option<Value>> {
        Ok(self.0.get(&key).cloned())
    }

    fn put(&mut self, key: DBKey, value: Value) -> PmtreeResult<()> {
        self.0.insert(key, value);

        Ok(())
    }

    fn put_batch(&mut self, subtree: HashMap<DBKey, Value>) -> PmtreeResult<()> {
        self.0.extend(subtree);

        Ok(())
    }

    fn close(&mut self) -> PmtreeResult<()> {
        Ok(())
    }
}

impl Hasher for MyKeccak {
    type Fr = [u8; 32];

    fn default_leaf() -> Self::Fr {
        [0; 32]
    }

    fn serialize(value: Self::Fr) -> Value {
        value.to_vec()
    }

    fn deserialize(value: Value) -> Self::Fr {
        value.try_into().unwrap()
    }

    fn hash(input: &[Self::Fr]) -> Self::Fr {
        let mut output = [0; 32];
        let mut hasher = Keccak::v256();
        for element in input {
            hasher.update(element);
        }
        hasher.finalize(&mut output);
        output
    }
}

fn main() {
    let mut mt = MerkleTree::<MemoryDB, MyKeccak>::new(2, MemoryDBConfig).unwrap();

    assert_eq!(mt.capacity(), 4);
    assert_eq!(mt.depth(), 2);

    mt.update_next(hex!(
        "c1ba1812ff680ce84c1d5b4f1087eeb08147a4d510f3496b2849df3a73f5af95"
    ))
    .unwrap();
    // closes the connection to the database
    mt.close().unwrap();
}
```
