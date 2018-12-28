use crate::{schema::users, AppState};
use actix_web::{HttpResponse, Json, Path, Responder, Result, State};
use actix_web_async_await::await;
use diesel::prelude::*;
use serde::Serialize;

#[derive(Serialize, Queryable)]
struct User {
    id: i32,
    name: String,
}

pub async fn fetch(state: State<AppState>) -> Result<impl Responder> {
    let results = await!(state.db.get(|conn| users::table.load::<User>(conn)))?;

    Ok(Json(results))
}

#[derive(Insertable)]
#[table_name = "users"]
struct CreateUser {
    name: String,
}

pub async fn create(state: State<AppState>, name: Path<String>) -> Result<impl Responder> {
    await!(state.db.get(move |conn| diesel::insert_into(users::table)
        .values(CreateUser {
            name: name.into_inner()
        })
        .execute(conn)))?;

    Ok(HttpResponse::Created())
}
