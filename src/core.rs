use crate::error::Error;
use crate::models::{Location, LocationCommand, LocationQuery};

pub(crate) trait Mutex {
    fn multiple_acquire(indices: Vec<String>) -> Result<bool, Error>;
    fn multiple_release(indices: Vec<String>) -> Result<(), Error>;
    fn single_acquire(index: String) -> Result<bool, Error>;
    fn single_release(index: String) -> Result<(), Error>;
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
