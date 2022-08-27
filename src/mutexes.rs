use crate::core::Mutex;
use anyhow::Error;
use log::error;
use redis::{self, cluster::ClusterClient, Commands, ToRedisArgs};
use redlock::{Lock, RedLock};
use std::future::Future;
use std::pin::Pin;
use std::{fmt::Display, marker::PhantomData};
use tokio::time::{timeout, Duration};

#[derive(Clone)]
pub(crate) struct RedisMutex<'m> {
    client: RedLock,
    // 有效时长， 超过此时长视为已获取锁的线程超时未释放锁或者此锁在此有效时长内没有被获取
    expire: usize,
    // 获取锁的等待时长
    timeout: u64,
    phantom: PhantomData<Lock<'m>>,
}

pub(crate) trait RedisArg: ToRedisArgs + Display + Send + Sync {}

impl RedisArg for u64 {}

impl<T: RedisArg> RedisArg for &T {}

impl<'m> RedisMutex<'m> {
    pub fn new(client: RedLock, expire: usize, timeout: u64) -> Self {
        Self {
            client,
            expire,
            timeout,
            phantom: PhantomData,
        }
    }
    async fn acquire<K: RedisArg>(&self, key: &K) -> Result<Lock, Error> {
        let res = timeout(Duration::from_secs(self.timeout), async {
            loop {
                if let Some(l) = self.client.lock(format!("{}", key).as_bytes(), self.expire) {
                    return l;
                }
            }
        })
        .await?;
        Ok(res)
    }
}

impl<'a, K: RedisArg> Mutex<'a, K, Lock<'a>> for RedisMutex<'a> {
    fn multiple_acquire(&'a self, keys: Vec<K>) -> Pin<Box<dyn Future<Output = Result<Vec<Lock<'a>>, Error>> + 'a>>
    where
        K: 'a,
    {
        Box::pin(async move {
            let mut locks = Vec::new();
            for key in &keys {
                match self.acquire(key).await {
                    Ok(l) => locks.push(l),
                    Err(e) => {
                        error!("{e}");
                        break;
                    }
                }
            }
            if locks.len() != keys.len() {
                for l in &locks {
                    self.client.unlock(l);
                }
            }
            Ok(locks)
        })
    }

    fn multiple_release(&'a self, locks: Vec<Lock<'a>>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move {
            for lock in locks {
                self.client.unlock(&lock);
            }
            Ok(())
        })
    }

    fn single_acquire(&'a self, key: K) -> Pin<Box<dyn Future<Output = Result<Lock<'a>, Error>> + 'a>>
    where
        K: 'a,
    {
        Box::pin(async move {
            let lock = self.acquire(&key).await?;
            Ok(lock)
        })
    }

    fn single_release(&'a self, lock: Lock<'a>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move {
            self.client.unlock(&lock);
            Ok(())
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_release_script() {}

    #[tokio::test]
    async fn test_acquire() {}
}
