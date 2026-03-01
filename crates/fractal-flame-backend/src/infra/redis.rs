use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::{Config, Pool, Runtime};

#[derive(Clone)]
pub struct RedisPool {
    pool: Pool,
}

impl RedisPool {
    pub fn from_url(url: &str) -> Result<Self, RedisError> {
        let cfg = Config::from_url(url);
        let pool = cfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| RedisError::Pool(e.to_string()))?;
        Ok(Self { pool })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, RedisError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| RedisError::Pool(e.to_string()))?;
        conn.get(key).await.map_err(RedisError::Redis)
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<(), RedisError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| RedisError::Pool(e.to_string()))?;
        if let Some(ttl) = ttl_secs {
            conn.set_ex::<_, _, ()>(key, value, ttl)
                .await
                .map_err(RedisError::Redis)?;
        } else {
            conn.set::<_, _, ()>(key, value)
                .await
                .map_err(RedisError::Redis)?;
        }
        Ok(())
    }

    pub async fn ping(&self) -> Result<(), RedisError> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|e| RedisError::Pool(e.to_string()))?;
        let _: () = deadpool_redis::redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(RedisError::Redis)?;
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RedisError {
    #[error("Redis pool error: {0}")]
    Pool(String),
    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),
}
