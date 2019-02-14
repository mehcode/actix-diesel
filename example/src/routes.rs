use crate::{schema::users, AppState};
use actix_diesel::{dsl::AsyncRunQueryDsl, AsyncError};
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    HttpResponse, Json, Path, Responder, Result, State,
};
use actix_web_async_await::await;
use diesel::{prelude::*, result::Error};
use futures::future::Future;
use serde::Serialize;

#[derive(Serialize, Queryable)]
struct User {
    id: i32,
    name: String,
}

pub async fn fetch_all(state: State<AppState>) -> Result<impl Responder> {
    let results = await!(users::table.load_async::<User>(&state.db))?;

    Ok(Json(results))
}

pub async fn fetch_one(state: State<AppState>, name: Path<String>) -> Result<impl Responder> {
    let result = await!(users::table
        .filter(users::name.eq(name.into_inner()))
        .get_result_async::<User>(&state.db)
        .map_err(|err| match err {
            AsyncError::Execute(Error::NotFound) => ErrorNotFound(err),
            _ => ErrorInternalServerError(err),
        }))?;

    Ok(Json(result))
}

#[derive(Insertable)]
#[table_name = "users"]
struct CreateUser {
    name: String,
}

pub async fn create(state: State<AppState>, name: Path<String>) -> Result<impl Responder> {
    await!(diesel::insert_into(users::table)
        .values(CreateUser {
            name: name.into_inner()
        })
        .execute_async(&state.db))?;

    Ok(HttpResponse::Created())
}
