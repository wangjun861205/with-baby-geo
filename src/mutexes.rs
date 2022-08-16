use crate::core::Mutex;
use anyhow::Error;
use redis::{self, cluster::ClusterClient, Commands, ToRedisArgs};
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

// KEYS[1]: is initiated
// KEYS[2]: seq num
// KEYS[3]: owner
// KEYS[4]: obtain time
// ARGV[1]: valid duration
const INIT_MUTEX_SCRIPT: &str = r#"
local is_inited = redis.call("GETDEL", KEYS[1]);
if not is_inited then
    local seq = redis.call("INCR", KEYS[2]);
    local now = redis.call("time")[1];
    redis.call("SET", KEYS[3], seq);
    redis.call("SET", KEYS[4], now);
    redis.call("SET", KEYS[1], "true");
    return true
end


redis.call("SET", KEYS[1], "true");
local prev_owner = redis.call("GETDEL", KEYS[3]);
local prev_time = redis.call("GETDEL", KEYS[4]);

local now = redis.call("time")[1];
if not prev_owner and not prev_time or tonumber(now, 10) - tonumber(prev_time, 10) >= tonumber(ARGV[1], 10) then
    local seq = redis.call("INCR", KEYS[2]);
    redis.call("SET", KEYS[3], seq);
    local now = redis.call("time")[1];
    redis.call("SET", KEYS[4], now);
    return true;
end

redis.call("SET", KEYS[3], prev_owner);
redis.call("SET", KEYS[4], prev_time);
return false;
"#;

// KEYS[1]: seq num
// KEYS[2]: owner
// KEYS[3]: obtain time
// KEYS[4]: queue
// ARGV[1]: valid duration
const RELEASE_MUTEX_SCRIPT: &str = r#"
local seq = redis.call("INCR", KEYS[1]);
local prev_owner = redis.call("GETDEL", KEYS[2]);
local prev_time = redis.call("GETDEL", KEYS[3]);
if not prev_owner or not prev_time then
    return
end
local now = redis.call("time")[1];
if seq - prev_owner == 1 or now - prev_time >= ARGV[1] then 
    redis.call("RPUSH", KEYS[4], "lock");
    return
end
redis.call("SET", KEYS[2], prev_owner);
redis.call("SET", KEYS[3], prev_time);
"#;

#[derive(Clone)]
pub(crate) struct RedisMutex {
    client: ClusterClient,
    // 有效时长， 超过此时长视为已获取锁的线程超时未释放锁或者此锁在此有效时长内没有被获取
    expire: usize,
    // 获取锁的等待时长
    timeout: usize,
}

pub(crate) trait RedisArg: ToRedisArgs + Display + Send + Sync {}

impl RedisArg for u64 {}

impl<T: RedisArg> RedisArg for &T {}

impl RedisMutex {
    pub fn new(client: ClusterClient, expire: usize, timeout: usize) -> Self {
        Self {
            client,
            expire,
            timeout,
        }
    }
    async fn acquire<K: RedisArg>(&self, key: K) -> Result<(), Error> {
        let init_key = format!("is_{}_initialized", key);
        let seq_key = format!("{}_seq", key);
        let owner_key = format!("{}_obtained_by", &key);
        let time_key = format!("{}_obtained_at", &key);
        let mut conn = self.client.get_connection()?;
        // KEYS[1]: is initiated
        // KEYS[2]: seq num
        // KEYS[3]: owner
        // KEYS[4]: obtain time
        // ARGV[1]: valid duration
        let res: bool = redis::Script::new(INIT_MUTEX_SCRIPT)
            .key(init_key)
            .key(seq_key)
            .key(owner_key)
            .key(time_key)
            .arg(self.expire as i32)
            .invoke(&mut conn)?;
        if res {
            return Ok(());
        }
        conn.brpop(key, self.timeout)?;
        return Ok(());
    }

    async fn release<K: RedisArg>(&self, key: K) -> Result<(), Error> {
        // KEYS[1]: seq num
        // KEYS[2]: owner
        // KEYS[3]: obtain time
        // KEYS[4]: queue
        // ARGV[1]: valid duration
        let seq_key = format!("{}_seq", key);
        let owner_key = format!("{}_obtained_by", &key);
        let time_key = format!("{}_obtained_at", &key);
        let queue_key = format!("{}_queue", key);
        let mut conn = self.client.get_connection()?;
        redis::Script::new(RELEASE_MUTEX_SCRIPT)
            .key(seq_key)
            .key(owner_key)
            .key(time_key)
            .key(queue_key)
            .arg(self.expire as i32)
            .invoke(&mut conn)?;
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

#[cfg(test)]
mod test {
    use super::*;

    // KEYS[1]: is initiated
    // KEYS[2]: seq num
    // KEYS[3]: owner
    // KEYS[4]: obtain time
    // ARGV[1]: valid duration
    #[tokio::test]
    async fn test_execute_lua_script() {
        let client = redis::Client::open("redis://localhost").unwrap();
        let mut conn = client.get_async_connection().await.unwrap();
        let res: bool = redis::Script::new(INIT_MUTEX_SCRIPT)
            .key("a_init")
            .key("a_seq")
            .key("a_owner")
            .key("a_time")
            .arg(60)
            .invoke_async(&mut conn)
            .await
            .unwrap();
        println!("{}", res)
    }

    // KEYS[1]: seq num
    // KEYS[2]: owner
    // KEYS[3]: obtain time
    // KEYS[4]: queue
    // ARGV[1]: valid duration
    #[tokio::test]
    async fn test_release_script() {
        let client = redis::Client::open("redis://localhost").unwrap();
        let mut conn = client.get_async_connection().await.unwrap();
        let res: bool = redis::Script::new(RELEASE_MUTEX_SCRIPT)
            .key("a_seq")
            .key("a_owner")
            .key("a_time")
            .key("a_queue")
            .arg(60)
            .invoke_async(&mut conn)
            .await
            .unwrap();
        println!("{}", res)
    }

    #[tokio::test]
    async fn test_acquire() {
        let client = redis::cluster::ClusterClient::open(vec!["redis://localhost"]).unwrap();
        let mutex = RedisMutex::new(client, 1, 3);
        mutex.acquire("test".to_owned()).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        mutex.release("test".to_owned()).await.unwrap();
    }
}
