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
use anyhow::Error;
use dotenv;
use indexers::H3Indexer;
use log::warn;
use mutexes::{RedisArg, RedisMutex};
use persisters::MongoPersister;
use std::env;

impl<'a> Key<'a> for i64 {}
impl RedisArg for i64 {}
impl RedisArg for String {}

fn init_redis_mutex() -> Result<RedisMutex<'static>, Error> {
    let uris = env::var("REDIS_URIS")?.split(",").map(str::to_owned).collect();
    let expire = env::var("REDIS_EXPIRE").unwrap_or("60".into()).parse::<usize>()?;
    let timeout = env::var("REDIS_TIMEOUT").unwrap_or("10".into()).parse::<u64>()?;
    let client = redlock::RedLock::new(uris);
    Ok(RedisMutex::new(client, expire, timeout))
}

async fn init_mongo_persister() -> Result<MongoPersister, Error> {
    let uris = env::var("MONGO_URIS")?;
    let database = env::var("MONGO_DATABASE")?;
    let db = mongodb::Client::with_options(mongodb::options::ClientOptions::parse(uris).await?)?.database(&database);
    Ok(MongoPersister::new(db))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    if let Err(e) = dotenv::dotenv() {
        if !e.not_found() {
            panic!("{e}");
        }
        warn!("cannot load .env: {e}");
    }
    let mutex = init_redis_mutex().expect("failed to init redis mutex");
    let indexer = H3Indexer::new(8).unwrap();
    let persister = init_mongo_persister().await.expect("failed to init mongo persister");
    let port = env::var("PORT").unwrap_or("8000".into());
    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .route(
                "/locations",
                post().to(add_location::<i64, H3Indexer, RedisMutex, MongoPersister, redlock::Lock>),
            )
            .route(
                "/locations",
                get().to(nearby_locations::<i64, H3Indexer, MongoPersister>),
            )
            .app_data(Data::new(mutex.clone()))
            .app_data(Data::new(indexer.clone()))
            .app_data(Data::new(persister.clone()))
    })
    .bind(format!("0.0.0.0:{port}"))
    .expect("failed to bind address")
    .run()
    .await
}
