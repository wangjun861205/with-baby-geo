use crate::core::Persister;
use crate::models::*;
use mongodb::options::FindOptions;

pub(crate) struct MongoPersister {
    coll: mongodb::Collection<Location<String>>,
}

impl Persister<String, String> for MongoPersister {
    fn insert<'a>(&'a mut self, loc: Location<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + 'a>> {
        Box::pin(async move {
            let res = self.coll.insert_one(loc, None).await?;
            Ok(res.inserted_id.to_string())
        })
    }

    fn all_by_indices(&mut self, indices: Vec<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Location<String>>, anyhow::Error>>>> {
        unimplemented!()
    }

    fn query_by_distance(
        &mut self,
        latitude: f64,
        longitude: f64,
        distance: f64,
        page: i32,
        size: i32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(Vec<Location<String>>, i64), anyhow::Error>>>> {
        unimplemented!()
    }
}
