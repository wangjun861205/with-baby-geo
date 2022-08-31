use crate::core::{self, Indexer, Key, Mutex, Persister};
use crate::error::Error;
use crate::models::Location;
use actix_web::{
    error::ParseError,
    http::header::{Header as ParseHeader, HeaderName, HeaderValue, InvalidHeaderValue, TryIntoHeaderValue},
    web::{Data, Header, Json, Query},
    HttpMessage,
};
use log::error;
use serde::{Deserialize, Serialize};

pub struct UID(String);

impl TryIntoHeaderValue for UID {
    type Error = InvalidHeaderValue;
    fn try_into_value(self) -> Result<HeaderValue, Self::Error> {
        HeaderValue::from_str(&self.0)
    }
}

impl ParseHeader for UID {
    fn name() -> HeaderName {
        HeaderName::from_static("X-UID")
    }
    fn parse<M: HttpMessage>(msg: &M) -> Result<Self, ParseError> {
        let uid = msg.headers().get(Self::name()).ok_or(ParseError::Header)?.to_str().map_err(|e| {
            error!("{}", e);
            ParseError::Header
        })?;
        Ok(UID(uid.to_owned()))
    }
}

#[derive(Deserialize)]
pub(crate) struct AddLocation {
    latitude: f64,
    longitude: f64,
}

pub(crate) async fn add_location<K, I, M, P, L>(Header(UID(uid)): Header<UID>, Json(loc): Json<AddLocation>, indexer: Data<I>, mutex: Data<M>, persister: Data<P>) -> Result<Json<String>, Error>
where
    K: Key<'static> + 'static,
    I: Indexer<'static, K> + Clone + 'static,
    M: Mutex<K, L> + Clone + 'static,
    P: Persister<K> + Clone + 'static,
    L: 'static,
{
    let res = core::add_location(mutex.get_ref().clone(), indexer.get_ref().clone(), persister.get_ref().clone(), loc.latitude, loc.longitude, 500.0, uid).await?;
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

pub(crate) async fn nearby_locations<'a, K, I, P>(Query(query): Query<NearbyLocation>, indexer: Data<I>, persister: Data<P>) -> Result<Json<NearbyLocationsResponse<K>>, Error>
where
    K: Key<'a> + 'a,
    I: Indexer<'a, K>,
    P: Persister<K>,
{
    let (locs, total) = core::nearby_locations(indexer.as_ref(), persister.as_ref(), query.latitude, query.longitude, 20000.0, query.page, query.size).await?;
    Ok(Json(NearbyLocationsResponse { list: locs, total: total }))
}
