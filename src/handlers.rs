use crate::core::{self, Indexer, Key, Mutex, Persister};
use crate::error::Error;
use crate::models::Location;
use actix_header::actix_header;
use actix_web::web::{Data, Header, Json, Query};
use serde::{Deserialize, Serialize};

#[actix_header("UID")]
pub struct UID(String);

impl From<String> for UID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<UID> for String {
    fn from(u: UID) -> Self {
        u.0
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

#[cfg(test)]
mod test {
    use super::*;
    use actix_header::actix_header;

    #[actix_header("X-CUSTOMIZED-HEADER")]
    struct MyCustomizedHeader(String);

    impl From<String> for MyCustomizedHeader {
        fn from(s: String) -> Self {
            Self(s)
        }
    }

    impl From<MyCustomizedHeader> for String {
        fn from(s: MyCustomizedHeader) -> Self {
            s.0
        }
    }

    #[test]
    fn test_actix_header() {
        let name = MyCustomizedHeader::name();
        println!("{}", name)
    }
}
