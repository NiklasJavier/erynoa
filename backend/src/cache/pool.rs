//! Cache Connection Pool

use anyhow::{Context, Result};
use fred::prelude::*;
use std::time::Duration;

use crate::config::CacheSettings;

#[derive(Clone)]
pub struct CachePool {
    client: RedisClient,
    default_ttl: Duration,
}

impl CachePool {
    pub async fn connect(settings: &CacheSettings) -> Result<Self> {
        let config = RedisConfig::from_url(&settings.url)
            .context("Invalid cache URL")?;

        let policy = ReconnectPolicy::new_exponential(0, 1000, 30_000, 2);

        let client = Builder::from_config(config)
            .with_performance_config(|c| {
                c.auto_pipeline = true;
                c.default_command_timeout = Duration::from_secs(5);
            })
            .with_connection_config(|c| {
                c.max_redirections = 3;
            })
            .set_policy(policy)
            .build()?;

        client.init().await.context("Failed to connect to cache")?;

        Ok(Self {
            client,
            default_ttl: Duration::from_secs(settings.default_ttl),
        })
    }

    pub async fn ping(&self) -> Result<()> {
        self.client
            .ping::<()>()
            .await
            .context("Cache ping failed")?;
        Ok(())
    }

    pub fn inner(&self) -> &RedisClient {
        &self.client
    }

    pub async fn get<T: FromRedis>(&self, key: &str) -> Result<Option<T>> {
        self.client.get(key).await.context("Cache get failed")
    }

    pub async fn set(&self, key: &str, value: impl Into<RedisValue>) -> Result<()> {
        self.client
            .set::<(), _, _>(
                key,
                value.into(),
                Some(Expiration::EX(self.default_ttl.as_secs() as i64)),
                None,
                false,
            )
            .await
            .context("Cache set failed")
    }

    pub async fn del(&self, key: &str) -> Result<()> {
        self.client.del::<(), _>(key).await.context("Cache delete failed")
    }
}

impl std::ops::Deref for CachePool {
    type Target = RedisClient;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
