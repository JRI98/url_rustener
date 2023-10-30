use anyhow::{anyhow, Context, Result};
use redis::{Client, Commands, ConnectionLike, ToRedisArgs};

pub struct KVDatabase {
    database: Client,
}

impl KVDatabase {
    pub fn new(url: &str) -> Result<KVDatabase> {
        let mut client = redis::Client::open(url)?;
        if !client.check_connection() {
            return Err(anyhow!("Could not connect to database"));
        };

        Ok(KVDatabase { database: client })
    }

    pub fn hget<K: ToRedisArgs, V: ToRedisArgs>(&self, key: K, field: V) -> Result<Option<String>> {
        let mut connection = self
            .database
            .get_connection()
            .context("Failed to get connection")?;
        let value = connection.hget(key, field).context("Failed to hget")?;

        Ok(value)
    }

    pub fn hset<K: ToRedisArgs, F: ToRedisArgs, V: ToRedisArgs>(
        &self,
        key: K,
        field: F,
        value: V,
    ) -> Result<()> {
        let mut connection = self
            .database
            .get_connection()
            .context("Failed to get connection")?;
        connection
            .hset(key, field, value)
            .context("Failed to hset")?;

        Ok(())
    }

    pub fn hincrby<K: ToRedisArgs, F: ToRedisArgs, D: ToRedisArgs>(
        &self,
        key: K,
        field: F,
        delta: D,
    ) -> Result<()> {
        let mut connection = self
            .database
            .get_connection()
            .context("Failed to get connection")?;
        connection
            .hincr(key, field, delta)
            .context("Failed to hincrby")?;

        Ok(())
    }

    pub fn del<K: ToRedisArgs>(&self, key: K) -> Result<()> {
        let mut connection = self
            .database
            .get_connection()
            .context("Failed to get connection")?;
        connection.del(key).context("Failed to del key")?;

        Ok(())
    }
}
