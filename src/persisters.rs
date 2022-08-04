use crate::core::Persister;
use crate::models::*;

pub(crate) struct MongoPersister {
    coll: mongodb::Collection<Location<String>>,
}

impl Persister<String> for MongoPersister {
    fn insert<'a>(&'a mut self, loc: Location<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, anyhow::Error>> + 'a>> {
        Box::pin(async move {
            let res = self.coll.insert_one(loc, None).await?;
            Ok(res.inserted_id.to_string())
        })
    }

    fn query(&mut self, query: LocationQuery<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(Vec<Location<String>>, i64), anyhow::Error>>>> {
        Box::pin(async move { Ok((Vec::new(), 0)) })
    }

    fn update(&mut self, id: &str, loc: LocationCommand<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<usize, anyhow::Error>>>> {
        Box::pin(async move { Ok(0) })
    }
}
