use crate::core::Mutex;
use anyhow::Error;
use log::error;
use redis::{self, ToRedisArgs};
use redlock::{Lock, RedLock};
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;
use tokio::time::{timeout, Duration};

pub struct MyLock {
    pub resource: Vec<u8>,
    pub val: Vec<u8>,
    pub validity_time: usize,
}

impl From<Lock<'_>> for MyLock {
    fn from(l: Lock) -> Self {
        Self {
            resource: l.resource,
            val: l.val,
            validity_time: l.validity_time,
        }
    }
}

impl MyLock {
    fn to_lock(self, rl: &RedLock) -> Lock {
        Lock {
            resource: self.resource,
            val: self.val,
            validity_time: self.validity_time,
            lock_manager: rl,
        }
    }
}

#[derive(Clone)]
pub(crate) struct RedisMutex {
    client: RedLock,
    // 有效时长， 超过此时长视为已获取锁的线程超时未释放锁或者此锁在此有效时长内没有被获取
    expire: usize,
    // 获取锁的等待时长
    timeout: u64,
}

pub(crate) trait RedisArg: ToRedisArgs + Display + Send + Sync {}

impl RedisArg for u64 {}

impl<T: RedisArg> RedisArg for &T {}

impl RedisMutex {
    pub fn new(client: RedLock, expire: usize, timeout: u64) -> Self {
        Self { client, expire, timeout }
    }
    async fn acquire<K: RedisArg>(self, key: &K) -> Result<MyLock, Error> {
        let res = timeout(Duration::from_secs(self.timeout), async {
            loop {
                if let Some(l) = self.client.lock(format!("{}", key).as_bytes(), self.expire) {
                    return l;
                }
            }
        })
        .await?;
        Ok(res.into())
    }
}

impl<K: RedisArg + 'static> Mutex<K, MyLock> for RedisMutex {
    fn multiple_acquire(self, keys: Vec<K>) -> Pin<Box<dyn Future<Output = Result<Vec<MyLock>, Error>>>> {
        Box::pin(async move {
            let mut locks = Vec::new();
            for key in &keys {
                match self.clone().acquire(key).await {
                    Ok(l) => locks.push(l),
                    Err(e) => {
                        error!("{e}");
                        break;
                    }
                }
            }
            if locks.len() != keys.len() {
                for l in locks {
                    self.client.clone().unlock(&l.to_lock(&self.client));
                }
                return Err(Error::msg("failed to get lock"));
            }
            Ok(locks)
        })
    }

    fn multiple_release(self, locks: Vec<MyLock>) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        Box::pin(async move {
            for lock in locks {
                self.client.unlock(&lock.to_lock(&self.client));
            }
            Ok(())
        })
    }

    fn single_acquire(self, key: K) -> Pin<Box<dyn Future<Output = Result<MyLock, Error>>>> {
        Box::pin(async move {
            let lock = self.acquire(&key).await?;
            Ok(lock)
        })
    }

    fn single_release(self, lock: MyLock) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        Box::pin(async move {
            self.client.unlock(&lock.to_lock(&self.client));
            Ok(())
        })
    }
}
