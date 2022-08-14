mod core;
mod error;
mod handlers;
mod indexers;
mod models;
mod mutexes;
mod persisters;

use crate::core::Key;

use crate::handlers::{add_location, nearby_locations};
use actix_web::{
    self,
    web::{get, post, Data},
};
use indexers::H3Indexer;
use mutexes::{RedisArg, RedisMutex};
use persisters::MongoPersister;

impl<'a> Key<'a> for i64 {}
impl RedisArg for i64 {}
impl RedisArg for String {}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mutex = RedisMutex::new(redis::Client::open("redis://localhost").unwrap(), 60, 5);
    let indexer = H3Indexer::new(8).unwrap();
    let persister: MongoPersister = MongoPersister::new(
        mongodb::Client::with_options(
            mongodb::options::ClientOptions::parse("mongodb://localhost")
                .await
                .unwrap(),
        )
        .unwrap()
        .database("with-baby-geo"),
    );
    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .route(
                "/locations",
                post().to(add_location::<i64, H3Indexer, RedisMutex, MongoPersister>),
            )
            .route(
                "/locations",
                get().to(nearby_locations::<i64, H3Indexer, MongoPersister>),
            )
            .app_data(Data::new(mutex.clone()))
            .app_data(Data::new(indexer.clone()))
            .app_data(Data::new(persister.clone()))
    })
    .bind("0.0.0.0:8000")
    .expect("failed to bind address")
    .run()
    .await
}
