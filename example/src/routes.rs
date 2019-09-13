use crate::{schema::users, AppState};
use actix_diesel::{dsl::AsyncRunQueryDsl, AsyncError};
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    web::{Data, Json, Path},
    HttpResponse, Responder, Result,
};
use diesel::{prelude::*, result::Error};
use futures::{compat::Future01CompatExt, future::TryFutureExt};
use serde::Serialize;

#[derive(Serialize, Queryable)]
struct User {
    id: i32,
    name: String,
}

pub async fn fetch_all(state: Data<AppState>) -> Result<impl Responder> {
    let results = users::table.load_async::<User>(&state.db).compat().await?;

    Ok(Json(results))
}

pub async fn fetch_one(state: Data<AppState>, name: Path<String>) -> Result<impl Responder> {
    let result = users::table
        .filter(users::name.eq(name.into_inner()))
        .get_result_async::<User>(&state.db)
        .compat()
        .map_err(|err| match err {
            AsyncError::Execute(Error::NotFound) => ErrorNotFound(err),
            _ => ErrorInternalServerError(err),
        })
        .await?;

    Ok(Json(result))
}

#[derive(Insertable)]
#[table_name = "users"]
struct CreateUser {
    name: String,
}

pub async fn create(state: Data<AppState>, name: Path<String>) -> Result<impl Responder> {
    diesel::insert_into(users::table)
        .values(CreateUser {
            name: name.into_inner(),
        })
        .execute_async(&state.db)
        .compat()
        .await?;

    Ok(HttpResponse::Created())
}
