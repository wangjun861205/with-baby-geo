use crate::core::Persister;
use crate::models::*;
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, from_document, Bson, Document},
    options::FindOptions,
};
use serde::Deserialize;
use tokio;

#[derive(Clone)]
pub(crate) struct MongoPersister {
    coll: mongodb::Collection<Document>,
}

impl MongoPersister {
    pub(crate) fn new(coll: mongodb::Collection<Document>) -> Self {
        Self { coll }
    }
}

impl<I> Persister<I> for MongoPersister
where
    for<'de> I: Into<Bson> + Deserialize<'de>,
{
    fn insert<'a>(
        &'a self,
        loc: LocationCommand<I>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + 'a>>
    where
        I: 'a,
    {
        Box::pin(async move {
            let res = self
                .coll
                .insert_one(
                    doc! {
                        "geo_index": loc.geo_index.into(),
                        "location": doc!{ "type": "Point", "coordinates": vec![loc.longitude, loc.latitude]}
                    },
                    None,
                )
                .await?;
            Ok(res.inserted_id.to_string())
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
            let mut res = self
                .coll
                .find(
                    condition.clone(),
                    FindOptions::builder()
                        .limit(size)
                        .skip((page as u64 - 1) * size as u64)
                        .build(),
                )
                .await?;
            let count = self.coll.count_documents(condition, None).await?;
            let mut l = Vec::new();
            while let Some(v) = res.try_next().await? {
                let loc: Location<I> = from_document(v)?;
                l.push(loc);
            }
            Ok((l, count))
        })
    }

    fn exists<'a>(
        &'a self,
        indices: Vec<I>,
        latitude: f64,
        longitude: f64,
        distance: f64,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, anyhow::Error>> + 'a>>
    where
        I: 'a,
    {
        Box::pin(async move {
            let count = self
                .coll
                .count_documents(
                    doc! {
                        "$and": vec![
                            doc!{ "geo_index": doc!{ "$in": indices }},
                            doc!{ "location": {
                                "$near": doc!{
                                "geometry": {
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
        let coll = mongodb::Client::with_options(ClientOptions::parse("mongodb://localhost:27017").await.unwrap())
            .unwrap()
            .database("with_baby_geo")
            .collection("locations");
        let p = MongoPersister::new(coll);
        let res = p
            .insert(LocationCommand {
                latitude: 36.657004,
                longitude: 117.0242607,
                geo_index: 613362111795429375i64,
            })
            .await
            .unwrap();
        println!("{}", res);
    }
}
