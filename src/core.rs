use crate::models::{Location, LocationCommand, LocationQuery};
use anyhow::Error;
use serde::Serialize;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

pub(crate) trait Key: Serialize + Display + Send + Sync {}

pub(crate) trait Mutex<K> {
    fn multiple_acquire<'a>(&'a mut self, keys: Vec<K>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a;
    fn multiple_release<'a>(&'a mut self, keys: Vec<K>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a;
    fn single_acquire<'a>(&'a mut self, key: K) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a;
    fn single_release<'a>(&'a mut self, key: K) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a;
}

pub(crate) trait Indexer<I>
where
    I: std::fmt::Display + Send + Sync,
{
    fn index(&self, latitude: f64, longitude: f64) -> I;
    fn neighbors(&self, index: I, distance: f64) -> Vec<I>;
}

pub(crate) trait Persister<I>
where
    I: Serialize,
{
    fn insert<'a>(&'a mut self, loc: Location<I>) -> Pin<Box<dyn Future<Output = Result<String, Error>> + 'a>>;
    fn query(&mut self, indices: Vec<I>, latitude: f64, longitude: f64, distance: f64, page: i32, size: i32) -> Pin<Box<dyn Future<Output = Result<(Vec<Location<I>>, i64), Error>>>>;
    fn exists<'a>(&'a mut self, indices: Vec<I>, latitude: f64, longitude: f64, distance: f64) -> Pin<Box<dyn Future<Output = Result<bool, Error>> + 'a>>;
}

pub(crate) async fn add_location<M, I, P, D, K, V>(mutex: &mut M, indexer: I, persister: &mut P, latitude: f64, longitude: f64, distance: f64) -> Result<String, Error>
where
    M: Mutex<K>,
    I: Indexer<K>,
    P: Persister<K>,
    K: Serialize + Display + Send + Sync + Clone + Ord,
{
    let idx = indexer.index(latitude, longitude);
    let mut neighbors = indexer.neighbors(idx.clone(), distance);
    neighbors.sort();
    mutex.multiple_acquire(neighbors.clone()).await?;
    if persister.exists(neighbors.clone(), latitude, longitude, distance).await? {
        mutex.multiple_release(neighbors.clone()).await?;
        return Err(Error::msg("already exists location nearby"));
    }
    let res = persister
        .insert(Location {
            latitude: latitude,
            longitude: longitude,
            geo_index: idx,
        })
        .await?;
    mutex.multiple_release(neighbors.clone()).await?;
    Ok(res)
}

pub(crate) async fn nearby_locations<I, P, K>(indexer: &I, persister: &mut P, latitude: f64, longitude: f64, distance: f64, page: i32, size: i32) -> Result<(Vec<Location<K>>, i64), Error>
where
    I: Indexer<K>,
    P: Persister<K>,
    K: Key,
{
    let idx = indexer.index(latitude, longitude);
    let indices = indexer.neighbors(idx, distance);
    let (locs, total) = persister.query(indices, latitude, longitude, distance, page, size).await?;
    Ok((locs, total))
}
