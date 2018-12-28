#![feature(async_await, await_macro, futures_api)]
#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use actix_diesel::Database;
use actix_web::http::Method;
use actix_web::{middleware, server, App};
use actix_web_async_await::{compat, compat2};
use diesel::sqlite::SqliteConnection;
use failure::Error;
use std::time::Duration;

mod routes;
mod schema;

use self::routes::*;

pub struct AppState {
    db: Database<SqliteConnection>,
}

fn main() -> Result<(), Error> {
    dotenv::dotenv()?;
    pretty_env_logger::try_init_timed()?;

    let db = Database::builder()
        // Maximum number of connections managed by the pool (default: 10)
        .pool_max_size(10)
        // Minimum idle connection count maintained by the pool (default: None {max_size})
        .pool_min_idle(Some(0))
        // Maximum lifetime of connections in the pool (default: 30 minutes)
        .pool_max_lifetime(Some(Duration::from_secs(30 * 60)))
        .open("file:example.sqlite");

    server::new(move || {
        App::with_state(AppState { db: db.clone() })
            .middleware(middleware::Logger::default())
            .route("/", Method::GET, compat(fetch))
            .route("/{name}", Method::POST, compat2(create))
    })
    .bind("127.0.0.1:8080")?
    .run();

    Ok(())
}
