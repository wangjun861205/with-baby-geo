use crate::core::Persister;
use crate::models::*;
use futures::{StreamExt, TryStreamExt};
use mongodb::{
    bson::{doc, from_document, oid::ObjectId, Bson, Document},
    options::FindOptions,
    Cursor,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct GeoJSON {
    #[serde(rename(deserialize = "type"))]
    typ: String,
    coordinates: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LocationIntermediate<I> {
    _id: ObjectId,
    geo_index: I,
    location: GeoJSON,
    uid: String,
}

#[derive(Clone)]
pub(crate) struct MongoPersister {
    db: mongodb::Database,
}

impl MongoPersister {
    pub(crate) fn new(db: mongodb::Database) -> Self {
        Self { db }
    }
}

impl<I> Persister<I> for MongoPersister
where
    for<'de> I: Into<Bson> + Deserialize<'de>,
{
    fn insert<'a>(&'a self, loc: LocationCommand<I>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + 'a>>
    where
        I: 'a,
    {
        Box::pin(async move {
            let res = self
                .db
                .collection("locations")
                .insert_one(
                    doc! {
                        "geo_index": loc.geo_index.into(),
                        "location": doc!{ "type": "Point", "coordinates": vec![loc.longitude, loc.latitude], "uid": loc.uid}
                    },
                    None,
                )
                .await?;

            Ok(res.inserted_id.as_object_id().unwrap().to_hex())
        })
    }

    fn query<'a>(
        &'a self,
        indices: Vec<I>,
        latitude: f64,
        longitude: f64,
        distance: f64,
        page: i64,
        size: i64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(Vec<Location<I>>, u64), anyhow::Error>> + 'a>>
    where
        I: 'a,
    {
        Box::pin(async move {
            let condition = doc! {"$and": vec![
                doc!{"geo_index": doc!{ "$in": indices }},
                doc!{"location":
                        {
                            "$near": {
                                "$geometry": {
                                    "type": "Point",
                                    "coordinates": vec![longitude, latitude]
                                },
                                "$maxDistance": distance
                            }
                        }
                    }
            ]};
            let mut res: Cursor<Document> = self
                .db
                .collection("locations")
                .find(condition.clone(), FindOptions::builder().limit(size).skip((page as u64 - 1) * size as u64).build())
                .await?;
            let count = self.db.run_command(doc! {"count": "locations", "query": condition}, None).await?.get_i32("n")?;
            let mut l = Vec::new();
            while let Some(v) = res.try_next().await? {
                let loc_im: LocationIntermediate<I> = from_document(v)?;
                let loc = Location::<I> {
                    id: loc_im._id.to_string(),
                    geo_index: loc_im.geo_index,
                    latitude: loc_im.location.coordinates[1],
                    longitude: loc_im.location.coordinates[0],
                };
                l.push(loc);
            }
            Ok((l, count as u64))
        })
    }

    fn exists<'a>(&'a self, indices: Vec<I>, latitude: f64, longitude: f64, distance: f64) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, anyhow::Error>> + 'a>>
    where
        I: 'a,
    {
        Box::pin(async move {
            let res = self
                .db
                .collection::<Document>("locations")
                .find(
                    doc! {
                            "$and": vec![
                                doc!{ "geo_index": doc!{ "$in": indices }},
                                doc!{ "location": {
                                    "$near": doc!{
                                    "$geometry": {
                                        "type": "Point",
                                        "coordinates": vec![longitude, latitude],
                                    },
                                    "$maxDistance": distance
                                }}}
                            ]
                    },
                    None,
                )
                .await?;
            let count = res.count().await;
            Ok(count > 0)
        })
    }
}

#[cfg(test)]
mod test {
    use mongodb::options::ClientOptions;

    use super::*;
    #[tokio::test]
    async fn test_insert() {
        let db = mongodb::Client::with_options(ClientOptions::parse("mongodb://localhost:27017").await.unwrap())
            .unwrap()
            .database("with-baby-geo");
        let p = MongoPersister::new(db);
        let res = p
            .insert(LocationCommand {
                latitude: 36.657004,
                longitude: 117.0242607,
                geo_index: 613362111795429375i64,
                uid: "1".into(),
            })
            .await
            .unwrap();
        println!("{}", res);
    }

    #[tokio::test]
    async fn test_exists() {
        let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
        client_options.app_name = Some("with-baby-geo".to_owned());
        let db = mongodb::Client::with_options(client_options).unwrap().database("with_baby_geo");
        let p = MongoPersister::new(db);
        let res = p.exists(vec![613362111795429375i64], 36.65, 117.02, 100000.0).await.unwrap();
        println!("{}", res);
    }

    #[tokio::test]
    async fn test_database_level_exists() {
        let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
        client_options.app_name = Some("with-baby-geo".to_owned());
        let db = mongodb::Client::with_options(client_options).unwrap().database("with_baby_geo");
        let res = db
            .run_command(
                doc! {
                    "count": "locations",
                    "query": doc!{
                            "$and": vec![
                                doc!{ "geo_index": doc!{ "$in": vec![613362111795429375i64] }},
                                doc!{ "location": {
                                    "$near": doc!{
                                    "$geometry": {
                                        "type": "Point",
                                        "coordinates": vec![117, 36],
                                    },
                                    "$maxDistance": 100000
                                }}}
                            ]
                    }
                },
                None,
            )
            .await
            .unwrap();
        let count = res.get_i32("n").unwrap();
        println!("{}", count);
    }
}
