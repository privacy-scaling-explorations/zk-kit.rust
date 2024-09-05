use hex_literal::hex;
use std::collections::HashMap;
use tiny_keccak::{Hasher as _, Keccak};
use zk_kit_pmt::*;

struct MemoryDB(HashMap<DBKey, Value>);
struct MyKeccak(Keccak);

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
        self.0.extend(subtree.into_iter());

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

#[test]
fn insert_delete() -> PmtreeResult<()> {
    let mut mt = MerkleTree::<MemoryDB, MyKeccak>::new(2, MemoryDBConfig)?;

    assert_eq!(mt.capacity(), 4);
    assert_eq!(mt.depth(), 2);

    let leaves = [
        hex!("0000000000000000000000000000000000000000000000000000000000000001"),
        hex!("0000000000000000000000000000000000000000000000000000000000000002"),
        hex!("0000000000000000000000000000000000000000000000000000000000000003"),
        hex!("0000000000000000000000000000000000000000000000000000000000000004"),
    ];

    let default_tree_root =
        hex!("b4c11951957c6f8f642c4af61cd6b24640fec6dc7fc607ee8206a99e92410d30");

    assert_eq!(mt.root(), default_tree_root);

    let roots = [
        hex!("c1ba1812ff680ce84c1d5b4f1087eeb08147a4d510f3496b2849df3a73f5af95"),
        hex!("893760ec5b5bee236f29e85aef64f17139c3c1b7ff24ce64eb6315fca0f2485b"),
        hex!("222ff5e0b5877792c2bc1670e2ccd0c2c97cd7bb1672a57d598db05092d3d72c"),
        hex!("a9bb8c3f1f12e9aa903a50c47f314b57610a3ab32f2d463293f58836def38d36"),
    ];

    for i in 0..leaves.len() {
        mt.update_next(leaves[i])?;
        assert_eq!(mt.root(), roots[i]);
    }

    for (i, &leaf) in leaves.iter().enumerate() {
        assert!(mt.verify(&leaf, &mt.proof(i)?));
    }

    for i in (0..leaves.len()).rev() {
        mt.delete(i)?;
    }

    assert_eq!(mt.root(), default_tree_root);

    assert!(mt.update_next(leaves[0]).is_err());

    Ok(())
}

#[test]
fn batch_insertions() -> PmtreeResult<()> {
    let mut mt = MerkleTree::<MemoryDB, MyKeccak>::new(2, MemoryDBConfig)?;

    let leaves = [
        hex!("0000000000000000000000000000000000000000000000000000000000000001"),
        hex!("0000000000000000000000000000000000000000000000000000000000000002"),
        hex!("0000000000000000000000000000000000000000000000000000000000000003"),
        hex!("0000000000000000000000000000000000000000000000000000000000000004"),
    ];

    mt.batch_insert(None, &leaves)?;

    assert_eq!(
        mt.root(),
        hex!("a9bb8c3f1f12e9aa903a50c47f314b57610a3ab32f2d463293f58836def38d36")
    );

    Ok(())
}

#[test]
fn set_range() -> PmtreeResult<()> {
    let mut mt = MerkleTree::<MemoryDB, MyKeccak>::new(2, MemoryDBConfig)?;

    let leaves = [
        hex!("0000000000000000000000000000000000000000000000000000000000000001"),
        hex!("0000000000000000000000000000000000000000000000000000000000000002"),
    ];

    mt.set_range(2, leaves)?;

    assert_eq!(
        mt.root(),
        hex!("1e9f6c8d3fd5b7ae3a29792adb094c6d4cc6149d0c81c8c8e57cf06c161a92b8")
    );

    Ok(())
}
