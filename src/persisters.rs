use crate::core::Persister;
use crate::models::*;
use serde::Serialize;

pub(crate) struct MongoPersister<I>
where
    I: Serialize,
{
    coll: mongodb::Collection<Location<I>>,
}

impl<I> Persister<I> for MongoPersister<I>
where
    I: Serialize,
{
    fn insert<'a>(&'a mut self, loc: Location<I>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + 'a>> {
        Box::pin(async move {
            let res = self.coll.insert_one(loc, None).await?;
            Ok(res.inserted_id.to_string())
        })
    }

    fn query(
        &mut self,
        indices: Vec<I>,
        latitude: f64,
        longitude: f64,
        distance: f64,
        page: i32,
        size: i32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(Vec<Location<I>>, i64), anyhow::Error>>>> {
        unimplemented!()
    }

    fn exists<'a>(&'a mut self, indices: Vec<I>, latitude: f64, longitude: f64, distance: f64) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, anyhow::Error>> + 'a>> {
        unimplemented!()
    }
}
