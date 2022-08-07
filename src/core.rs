use crate::models::{Location, LocationCommand};
use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

pub(crate) trait Key<'a>: Serialize + Deserialize<'a> + Display + Send + Sync + Ord + Clone {}

impl<'a> Key<'a> for String {}
impl<'a> Key<'a> for u64 {}

pub(crate) trait Mutex<K> {
    fn multiple_acquire<'a>(&'a self, keys: &'a [K]) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a;
    fn multiple_release<'a>(&'a self, keys: &'a [K]) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a;
    fn single_acquire<'a>(&'a self, key: &'a K) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
    where
        K: 'a;
    fn single_release<'a>(&'a self, key: &'a K) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>
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

pub(crate) trait Persister<I> {
    fn insert<'a>(&'a self, loc: LocationCommand<I>) -> Pin<Box<dyn Future<Output = Result<String, Error>> + 'a>>
    where
        I: 'a;
    fn query<'a>(
        &'a self,
        indices: Vec<I>,
        latitude: f64,
        longitude: f64,
        distance: f64,
        page: i64,
        size: i64,
    ) -> Pin<Box<dyn Future<Output = Result<(Vec<Location<I>>, u64), Error>> + 'a>>
    where
        I: 'a;
    fn exists<'a>(
        &'a self,
        indices: Vec<I>,
        latitude: f64,
        longitude: f64,
        distance: f64,
    ) -> Pin<Box<dyn Future<Output = Result<bool, Error>> + 'a>>
    where
        I: 'a;
}

pub(crate) async fn add_location<'a, M, I, P, K>(
    mutex: &M,
    indexer: &I,
    persister: &P,
    latitude: f64,
    longitude: f64,
    distance: f64,
) -> Result<String, Error>
where
    M: Mutex<K>,
    I: Indexer<K>,
    P: Persister<K>,
    K: Key<'a>,
{
    let idx = indexer.index(latitude, longitude);
    let mut neighbors = indexer.neighbors(idx.clone(), distance);
    neighbors.sort();
    mutex.multiple_acquire(&neighbors).await?;
    if persister
        .exists(neighbors.clone(), latitude, longitude, distance)
        .await?
    {
        mutex.multiple_release(&neighbors).await?;
        return Err(Error::msg("already exists location nearby"));
    }
    let res = persister
        .insert(LocationCommand {
            latitude: latitude,
            longitude: longitude,
            geo_index: idx,
        })
        .await?;
    mutex.multiple_release(&neighbors).await?;
    Ok(res)
}

pub(crate) async fn nearby_locations<'a, I, P, K>(
    indexer: &I,
    persister: &P,
    latitude: f64,
    longitude: f64,
    distance: f64,
    page: i64,
    size: i64,
) -> Result<(Vec<Location<K>>, u64), Error>
where
    I: Indexer<K>,
    P: Persister<K>,
    K: Key<'a>,
{
    let idx = indexer.index(latitude, longitude);
    let indices = indexer.neighbors(idx, distance);
    let (locs, total) = persister
        .query(indices, latitude, longitude, distance, page, size)
        .await?;
    Ok((locs, total))
}
