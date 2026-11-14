use super::fingerprint::{FINGERPRINT_HEADER, FingerprintCheck};
use super::{IdempotencyAction, IdempotencyKey, RequestFingerprint};
use actix_web::HttpResponse;
use actix_web::body::to_bytes;
use actix_web::http::StatusCode;
use sqlx::{Executor, PgPool};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    action: IdempotencyAction,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let saved_response = sqlx::query!(
        r#"
        SELECT
            response_status_code as "response_status_code!",
            response_headers as "response_headers!: Vec<HeaderPairRecord>",
            response_body as "response_body!"
        FROM idempotency
        WHERE
          idempotency_key = $1 AND
          action = $2
        "#,
        idempotency_key.as_ref(),
        action.as_str()
    )
    .fetch_optional(pool)
    .await?;
    if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;
        let mut response = HttpResponse::build(status_code);
        for HeaderPairRecord { name, value } in r.response_headers {
            response.append_header((name, value));
        }
        Ok(Some(response.body(r.response_body)))
    } else {
        Ok(None)
    }
}

pub async fn save_response(
    mut transaction: Transaction<'static, Postgres>,
    idempotency_key: &IdempotencyKey,
    action: IdempotencyAction,
    fingerprint: &RequestFingerprint,
    mut http_response: HttpResponse,
) -> Result<HttpResponse, anyhow::Error> {
    // Stamp the request fingerprint so a later replay can be bound to this request.
    http_response
        .headers_mut()
        .insert(FINGERPRINT_HEADER, fingerprint.header_value());
    let (mut response_head, body) = http_response.into_parts();
    let body = to_bytes(body).await.map_err(|e| anyhow::anyhow!("{}", e))?;
    let status_code = response_head.status().as_u16() as i16;
    let headers = {
        let mut h = Vec::with_capacity(response_head.headers().len());
        for (name, value) in response_head.headers().iter() {
            let name = name.as_str().to_owned();
            let value = value.as_bytes().to_owned();
            h.push(HeaderPairRecord { name, value });
        }
        h
    };
    transaction
        .execute(sqlx::query_unchecked!(
            r#"
        UPDATE idempotency
        SET
            response_status_code = $3,
            response_headers = $4,
            response_body = $5
        WHERE
            idempotency_key = $1 AND
            action = $2
        "#,
            idempotency_key.as_ref(),
            action.as_str(),
            status_code,
            headers,
            body.as_ref()
        ))
        .await?;
    transaction.commit().await?;

    // The fingerprint is internal bookkeeping; never hand it to the caller.
    response_head.headers_mut().remove(FINGERPRINT_HEADER);
    let http_response = response_head.set_body(body).map_into_boxed_body();
    Ok(http_response)
}

#[allow(clippy::large_enum_variant)]
pub enum NextAction {
    // Return transaction for later usage
    StartProcessing(Transaction<'static, Postgres>),
    ReturnSavedResponse(HttpResponse),
    /// The key was first used for a different request; nothing was written.
    RequestMismatch,
}

pub async fn try_processing(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: Uuid,
    action: IdempotencyAction,
    fingerprint: &RequestFingerprint,
) -> Result<NextAction, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let query = sqlx::query!(
        r#"
        INSERT INTO idempotency (
            user_id,
            idempotency_key,
            action,
            created_at
        )
        VALUES ($1, $2, $3, now())
        ON CONFLICT DO NOTHING
        "#,
        user_id,
        idempotency_key.as_ref(),
        action.as_str()
    );
    let n_inserted_rows = transaction.execute(query).await?.rows_affected();
    if n_inserted_rows > 0 {
        Ok(NextAction::StartProcessing(transaction))
    } else {
        let mut saved_response = get_saved_response(pool, idempotency_key, action)
            .await?
            .ok_or_else(|| anyhow::anyhow!("We expected a saved response, we didn't find it"))?;
        let check = fingerprint.check(saved_response.headers().get(FINGERPRINT_HEADER));
        saved_response.headers_mut().remove(FINGERPRINT_HEADER);
        match check {
            FingerprintCheck::Mismatch => Ok(NextAction::RequestMismatch),
            // Legacy rows (no valid fingerprint) replay as before.
            FingerprintCheck::Match | FingerprintCheck::Legacy => {
                Ok(NextAction::ReturnSavedResponse(saved_response))
            }
        }
    }
}
