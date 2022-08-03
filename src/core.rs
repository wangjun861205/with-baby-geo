use crate::models::{Location, LocationCommand, LocationQuery};
use anyhow::Error;
use std::future::Future;
use std::pin::Pin;

pub(crate) trait Mutex {
    fn multiple_acquire<'a>(&'a mut self, keys: Vec<String>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;
    fn multiple_release<'a>(&'a mut self, keys: Vec<String>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;
    fn single_acquire<'a>(&'a mut self, key: String) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;
    fn single_release<'a>(&'a mut self, key: String) -> Pin<Box<dyn Future<Output = Result<(), Error>> + 'a>>;
}

pub(crate) trait Indexer {
    fn index(latitude: f64, longitude: f64) -> String;
    fn neighbors(index: String, distance: f64) -> Vec<String>;
}

pub(crate) trait Persister<T> {
    fn insert(loc: LocationCommand) -> Result<T, Error>;
    fn update(id: T, loc: LocationCommand) -> Result<usize, Error>;
    fn query(query: LocationQuery) -> Result<(Vec<Location<T>>, i64), Error>;
}
