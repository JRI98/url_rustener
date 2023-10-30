use anyhow::{Context, Result};
use rocksdb::DB;
use serde::{de::DeserializeOwned, Serialize};

pub struct KVDatabase {
    database: DB,
}

impl KVDatabase {
    pub fn new() -> Result<KVDatabase> {
        let path = ".data";
        let db = DB::open_default(path)?;
        Ok(KVDatabase { database: db })
    }

    pub fn get<K: Serialize, V: DeserializeOwned>(&self, key: &K) -> Result<Option<V>> {
        let key_serialized = bincode::serialize(&key).context("Failed to serialize key")?;

        let value_serialized = match self
            .database
            .get(key_serialized)
            .context("Failed to get key")?
        {
            Some(vs) => vs,
            None => return Ok(None),
        };

        let value =
            bincode::deserialize(&value_serialized).context("Failed to deserialize value")?;
        Ok(Some(value))
    }

    pub fn put<K: Serialize, V: Serialize>(&self, key: &K, value: &V) -> Result<()> {
        let key_serialized = bincode::serialize(&key).context("Failed to serialize key")?;
        let value_serialized = bincode::serialize(&value).context("Failed to serialize value")?;

        self.database
            .put(key_serialized, value_serialized)
            .context("Failed to put key/value")?;

        Ok(())
    }

    pub fn delete<K: Serialize>(&self, key: &K) -> Result<()> {
        let key_serialized = bincode::serialize(&key).context("Failed to serialize key")?;

        self.database
            .delete(key_serialized)
            .context("Failed to put key/value")?;

        Ok(())
    }
}
