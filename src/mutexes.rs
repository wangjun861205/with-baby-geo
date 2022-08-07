use crate::core::Mutex;
use anyhow::Error;
use chrono::Utc;
use redis::{self, AsyncCommands, Client, ToRedisArgs};
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub(crate) struct RedisMutex {
    client: Client,
    // 有效时长， 超过此时长视为已获取锁的线程超时未释放锁或者此锁在此有效时长内没有被获取
    expire: usize,
    // 获取锁的等待时长
    timeout: usize,
}

pub(crate) trait RedisArg: ToRedisArgs + Display + Send + Sync {}

impl RedisArg for u64 {}

impl<T: RedisArg> RedisArg for &T {}

impl RedisMutex {
    pub fn new(client: Client, expire: usize, timeout: usize) -> Self {
        Self {
            client,
            expire,
            timeout,
        }
    }
    async fn acquire<K: RedisArg>(&self, key: K) -> Result<(), Error> {
        let init_key = format!("is_{}_initialized", key);
        let init_time_key = format!("{}_initialized_at", &key);
        let mut conn = self.client.get_async_connection().await?;
        let init: i32 = conn.set_nx(&init_key, true).await?;
        // 如果设置初始化位成功， 则直接设置初始化时间戳
        if init == 1 {
            conn.set(init_time_key, Utc::now().timestamp()).await?;
            return Ok(());
        }
        // 如果初始化位已经设置了， 则比对初始化时间戳
        let init_time: i64 = conn.get(&init_time_key).await?;
        // 如果超时了则重新设置初始化时间戳
        if Utc::now().timestamp() - init_time >= self.expire as i64 {
            let deleted: i32 = conn.del(&init_time_key).await?;
            if deleted == 1 {
                conn.set(&init_time_key, Utc::now().timestamp()).await?;
                return Ok(());
            }
        }
        // 如果没有超时或者其他task已经开始初始化时间戳，则等待其他线程释放锁
        conn.brpop(key, self.timeout).await?;
        return Ok(());
    }

    async fn release<K: RedisArg>(&self, key: K) -> Result<(), Error> {
        let init_time_key = format!("{}_initialized_at", &key);
        let mut conn = self.client.get_async_connection().await?;
        let deleted: i32 = conn.del(&init_time_key).await?;
        // 如果成功删除初始化时间戳, 则刷新时间戳并释放锁， 否则这两项工作需要其他成功删除时间戳的task来完成
        if deleted == 1 {
            conn.set(&init_time_key, Utc::now().timestamp()).await?;
            conn.rpush(&key, true).await?;
        }
        return Ok(());
    }
}

impl<K: RedisArg> Mutex<K> for RedisMutex {
    fn multiple_acquire<'a>(&'a self, keys: &'a [K]) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a,
    {
        Box::pin(async move {
            let mut i = 0;
            while i < keys.len() {
                if let Err(e) = self.acquire(&keys[i]).await {
                    for j in 0..i {
                        self.release(&keys[j]).await?;
                    }
                    return Err(e);
                }
                i += 1;
            }
            Ok(())
        })
    }

    fn multiple_release<'a>(&'a self, keys: &'a [K]) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a,
    {
        Box::pin(async move {
            for key in keys {
                self.release(key).await?;
            }
            Ok(())
        })
    }

    fn single_acquire<'a>(&'a self, key: &'a K) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a,
    {
        Box::pin(async move {
            self.acquire(key).await?;
            Ok(())
        })
    }

    fn single_release<'a>(&'a self, key: &'a K) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a,
    {
        Box::pin(async move {
            self.release(key).await?;
            Ok(())
        })
    }
}
