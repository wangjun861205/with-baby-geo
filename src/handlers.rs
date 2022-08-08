use crate::core::{self, Indexer, Key, Mutex, Persister};
use crate::error::Error;
use crate::models::Location;
use actix_web::web::{Data, Json, Query};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct AddLocation {
    latitude: f64,
    longitude: f64,
}

pub(crate) async fn add_location<K, I, M, P>(
    Json(loc): Json<AddLocation>,
    indexer: Data<I>,
    mutex: Data<M>,
    persister: Data<P>,
) -> Result<Json<String>, Error>
where
    for<'a> K: Key<'a>,
    I: Indexer<K>,
    M: Mutex<K>,
    P: Persister<K>,
{
    let res = core::add_location(
        mutex.as_ref(),
        indexer.as_ref(),
        persister.as_ref(),
        loc.latitude,
        loc.longitude,
        0.5,
    )
    .await?;
    Ok(Json(res))
}

#[derive(Deserialize)]
pub(crate) struct NearbyLocation {
    latitude: f64,
    longitude: f64,
    page: i64,
    size: i64,
}

#[derive(Serialize)]
pub(crate) struct NearbyLocationsResponse<I> {
    list: Vec<Location<I>>,
    total: u64,
}

pub(crate) async fn nearby_locations<K, I, P>(
    Query(query): Query<NearbyLocation>,
    indexer: Data<I>,
    persister: Data<P>,
) -> Result<Json<NearbyLocationsResponse<K>>, Error>
where
    for<'a> K: Key<'a>,
    I: Indexer<K>,
    P: Persister<K>,
{
    let (locs, total) = core::nearby_locations(
        indexer.as_ref(),
        persister.as_ref(),
        query.latitude,
        query.longitude,
        20.0,
        query.page,
        query.size,
    )
    .await?;
    Ok(Json(NearbyLocationsResponse {
        list: locs,
        total: total,
    }))
}
