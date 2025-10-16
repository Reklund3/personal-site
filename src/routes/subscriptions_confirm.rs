use crate::routes::error_chain_fmt;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{HttpResponse, ResponseError, web};
use anyhow::Context;
use sqlx::PgPool;
use std::fmt::{Debug, Formatter};
use uuid::Uuid;

#[derive(thiserror::Error)]
pub enum ConfirmSubscriptionError {
    #[error("{0}")]
    Unauthorized(String),
    #[error(transparent)]
    OtherError(#[from] anyhow::Error),
}

impl Debug for ConfirmSubscriptionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for ConfirmSubscriptionError {
    fn status_code(&self) -> StatusCode {
        match self {
            ConfirmSubscriptionError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ConfirmSubscriptionError::OtherError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber.", skip(pool))]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: Data<PgPool>,
) -> Result<HttpResponse, ConfirmSubscriptionError> {
    let maybe_subscriber_id =
        get_subscriber_id_from_token(parameters.subscription_token.as_str(), &pool)
            .await
            .context("Failed to retrieve the subscriber id from the data base.")
            .unwrap();

    match maybe_subscriber_id {
        Some(id) => confirm_subscriber(id, &pool).await,
        None => Err(ConfirmSubscriptionError::Unauthorized(
            "Subscriber id does not exists in postgres.".into(),
        )),
    }
    .context("Failed to retrieve a valid user id for the provided subscriber token.")?;
    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Marking the subscriber as confirmed.", skip(pool))]
async fn confirm_subscriber(
    subscriber_id: Uuid,
    pool: &PgPool,
) -> Result<(), ConfirmSubscriptionError> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(pool)
    .await
    .context("Failed to update the user record to confirmed.")?;
    Ok(())
}

#[tracing::instrument(
    name = "Fetching the subscriber id with the subscription token.",
    skip(pool)
)]
async fn get_subscriber_id_from_token(
    subscription_token: &str,
    pool: &PgPool,
) -> Result<Option<Uuid>, ConfirmSubscriptionError> {
    let result = sqlx::query!(
        r#"
            SELECT (subscriber_id) FROM subscription_tokens
            WHERE subscription_token = $1
        "#,
        subscription_token,
    )
    .fetch_optional(pool)
    .await
    .context("Failed to get the subscriber id from postgres.")?;
    Ok(result.map(|r| r.subscriber_id))
}
