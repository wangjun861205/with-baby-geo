use crate::models::{Location, LocationCommand, LocationQuery};
use anyhow::Error;
use serde::Serialize;
use std::fmt::Display;
use std::future::Future;
use std::pin::Pin;

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
    I: std::fmt::Display + Send + Sync + Clone,
{
    fn index(&self, latitude: f64, longitude: f64) -> I;
    fn neighbors(&self, index: I, distance: f64) -> Vec<I>;
}

pub(crate) trait Persister<I, V>
where
    I: Serialize,
{
    fn insert<'a>(&'a mut self, loc: Location<I>) -> Pin<Box<dyn Future<Output = Result<V, Error>> + 'a>>;
    fn all_by_indices(&mut self, indices: Vec<I>) -> Pin<Box<dyn Future<Output = Result<Vec<Location<I>>, Error>>>>;
    fn query_by_distance(&mut self, latitude: f64, longitude: f64, distance: f64, page: i32, size: i32) -> Pin<Box<dyn Future<Output = Result<(Vec<Location<I>>, i64), Error>>>>;
}

pub(crate) trait Distancer {
    fn distance(&self, src: (f64, f64), dst: (f64, f64)) -> f64;
}

pub(crate) async fn add_location<M, I, P, D, K, V>(mutex: &mut M, indexer: I, persister: &mut P, distancer: D, latitude: f64, longitude: f64, distance: f64) -> Result<V, Error>
where
    M: Mutex<K>,
    I: Indexer<K>,
    P: Persister<K, V>,
    D: Distancer,
    K: Serialize + Display + Send + Sync + Clone + Ord,
{
    let idx = indexer.index(latitude, longitude);
    let mut neighbors = indexer.neighbors(idx.clone(), distance);
    neighbors.sort();
    mutex.multiple_acquire(neighbors.clone()).await?;
    let locs = persister.all_by_indices(neighbors.clone()).await?;
    for loc in locs {
        let dist = distancer.distance((latitude, longitude), (loc.latitude, loc.longitude));
        if dist <= distance {
            mutex.multiple_release(neighbors.clone()).await?;
            return Err(Error::msg("already exists location nearby"));
        }
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
