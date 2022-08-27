use std::ops::Deref;

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

pub(crate) async fn add_location<'a, K, I, M, P, L>(
    Json(loc): Json<AddLocation>,
    indexer: Data<I>,
    mutex: Data<M>,
    persister: Data<P>,
) -> Result<Json<String>, Error>
where
    K: Key<'a> + 'a,
    I: Indexer<'a, K> + 'a,
    M: Mutex<'a, K, L> + 'a,
    P: Persister<K> + 'a,
    L: 'a,
{
    // let res = core::add_location(
    //     mutex.as_ref(),
    //     indexer.as_ref(),
    //     persister.as_ref(),
    //     loc.latitude,
    //     loc.longitude,
    //     0.5,
    // )
    // .await?;
    // Ok(Json(res))
    Ok(Json("Ok".into()))
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

pub(crate) async fn nearby_locations<'a, K, I, P>(
    Query(query): Query<NearbyLocation>,
    indexer: Data<I>,
    persister: Data<P>,
) -> Result<Json<NearbyLocationsResponse<K>>, Error>
where
    K: Key<'a> + 'a,
    I: Indexer<'a, K>,
    P: Persister<K>,
{
    let (locs, total) = core::nearby_locations(
        indexer.as_ref(),
        persister.as_ref(),
        query.latitude,
        query.longitude,
        20000.0,
        query.page,
        query.size,
    )
    .await?;
    Ok(Json(NearbyLocationsResponse {
        list: locs,
        total: total,
    }))
}
