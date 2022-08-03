use crate::core::Mutex;
use anyhow::Error;
use chrono::Utc;
use redis::{self, AsyncCommands};
use std::future::Future;
use std::pin::Pin;

pub(crate) struct RedisMutex {
    conn: redis::aio::Connection,
    // 有效时长， 超过此时长视为已获取锁的线程超时未释放锁或者此锁在此有效时长内没有被获取
    expire: usize,
    // 获取锁的等待时长
    timeout: usize,
}

impl RedisMutex {
    pub fn new(conn: redis::aio::Connection, expire: usize, timeout: usize) -> Self {
        Self { conn, expire, timeout }
    }
    async fn acquire(&mut self, key: &str) -> Result<(), Error> {
        let init_key = format!("is_{}_initialized", &key);
        let init_time_key = format!("{}_initialized_at", &key);
        let init: i32 = self.conn.set_nx(&init_key, true).await?;
        // 如果设置初始化位成功， 则直接设置初始化时间戳
        if init == 1 {
            self.conn.set(init_time_key, Utc::now().timestamp()).await?;
            return Ok(());
        }
        // 如果初始化位已经设置了， 则比对初始化时间戳
        let init_time: i64 = self.conn.get(&init_time_key).await?;
        // 如果超时了则重新设置初始化时间戳
        if Utc::now().timestamp() - init_time >= self.expire as i64 {
            let deleted: i32 = self.conn.del(&init_time_key).await?;
            if deleted == 1 {
                self.conn.set(&init_time_key, Utc::now().timestamp()).await?;
                return Ok(());
            }
        }
        // 如果没有超时或者其他task已经开始初始化时间戳，则等待其他线程释放锁
        self.conn.brpop(&key, self.timeout).await?;
        return Ok(());
    }

    async fn release(&mut self, key: &str) -> Result<(), Error> {
        let init_time_key = format!("{}_initialized_at", &key);
        let deleted: i32 = self.conn.del(&init_time_key).await?;
        // 如果成功删除初始化时间戳, 则刷新时间戳并释放锁， 否则这两项工作需要其他成功删除时间戳的task来完成
        if deleted == 1 {
            self.conn.set(&init_time_key, Utc::now().timestamp()).await?;
            self.conn.rpush(&key, true).await?;
        }
        return Ok(());
    }
}

impl Mutex for RedisMutex {
    fn multiple_acquire<'a>(&'a mut self, keys: Vec<String>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
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

    fn multiple_release<'a>(&'a mut self, keys: Vec<String>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move {
            for key in &keys {
                self.release(key).await?;
            }
            Ok(())
        })
    }

    fn single_acquire<'a>(&'a mut self, key: String) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move {
            self.acquire(&key).await?;
            Ok(())
        })
    }

    fn single_release<'a>(&'a mut self, key: String) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>> {
        Box::pin(async move {
            self.release(&key).await?;
            Ok(())
        })
    }
}
