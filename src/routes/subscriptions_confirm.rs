use std::{error::Error, fmt::Debug};

use actix_web::{HttpResponse, ResponseError, http::StatusCode, web};
use anyhow::Context;
use sqlx::{PgPool};
use uuid::Uuid;

use crate::routes::error_chain_fmt;


#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error("No subscriber found for given token")]
    UnknownTokenError
}

impl ResponseError for ConfirmError {
    fn status_code(&self) -> actix_web::http::StatusCode {

        match self {
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String
}

#[tracing::instrument(
    name = "Confirm a pending subscriber",
    skip(parameters, pool)
)]
pub async fn confirm(parameters: web::Query<Parameters>, pool: web::Data<PgPool>) -> Result<HttpResponse, ConfirmError> {

    let id = get_subscriber_id_from_token(
        &pool,
        &parameters.subscription_token
    ).await.context("Failed to get subscriber id from token")?
    .ok_or(ConfirmError::UnknownTokenError)?;

    confirm_subscriber(&pool, id).await.context("Failed to confirm subscriber")?;
    Ok(HttpResponse::Ok().finish())
       
}

#[tracing::instrument(
    name = "Mark subscriber as subscribed",
    skip(subscriber_id, pool)
)]
pub async fn confirm_subscriber(pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {

    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(pool)
    .await
    .map_err(|e| {
        e
    })?;

    Ok(())

}

pub async fn get_subscriber_id_from_token(pool: &PgPool, subscription_token: &str) -> Result<Option<Uuid>, sqlx::Error> {

    let result = sqlx::query!(
        "SELECT subscriber_id FROM subscription_tokens \
        WHERE subscription_token = $1",
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}