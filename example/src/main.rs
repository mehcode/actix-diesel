#[macro_use]
extern crate diesel;

use actix_diesel::Database;
use actix_web::{middleware, web, App, HttpServer};
use diesel::sqlite::SqliteConnection;
use failure::Error;
use futures::future::{FutureExt, TryFutureExt};
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

    HttpServer::new(move || {
        App::new()
            .data(AppState { db: db.clone() })
            .wrap(middleware::Logger::default())
            .route("/", web::get().to_async(|x| fetch_all(x).boxed().compat()))
            .route(
                "/{name}",
                web::get().to_async(|x, y| fetch_one(x, y).boxed().compat()),
            )
            .route(
                "/{name}",
                web::post().to_async(|x, y| create(x, y).boxed().compat()),
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .unwrap();

    Ok(())
}
