use crate::authentication::UserId;
use crate::domain::NewsletterIssueStatus;
use crate::idempotency::{IdempotencyKey, NextAction, save_response, try_processing};
use crate::issue_delivery_worker::mark_sent_if_drained;
use crate::utils::e400;
use crate::utils::{e500, see_other};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Persisted on the idempotency saved response so replay flash reflects the
/// first processing outcome, not a possibly different replay body.
const QUEUE_OUTCOME_HEADER: &str = "x-newsletter-queue-outcome";

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
    idempotency_key: String,
    /// Optional issue id. Absent ⇒ a new issue id is generated. Present ⇒ the
    /// issue is inserted under that id, or its content is updated while it is
    /// still a `draft`; either way it is then queued for delivery.
    #[serde(default)]
    newsletter_issue_id: Option<String>,
}

fn success_message() -> FlashMessage {
    FlashMessage::info(
        "The newsletter issue has been accepted - \
        emails will go out shortly.",
    )
}

fn already_handled_message() -> FlashMessage {
    FlashMessage::info(
        "This newsletter issue has already been handled \
        (queued, sent, or failed).",
    )
}

#[derive(Debug, PartialEq, Eq)]
pub enum QueueOutcome {
    /// Transitioned `draft` → `queued` and enqueued delivery rows.
    Queued,
    /// Issue exists but is already `queued` / `sent` / `failed` — do not enqueue again.
    AlreadyHandled { status: NewsletterIssueStatus },
}

#[tracing::instrument(
name = "Publish a newsletter issue",
skip_all,
fields(user_id=%&*user_id)
)]
pub async fn publish_newsletter(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let FormData {
        title,
        text_content,
        html_content,
        idempotency_key,
        newsletter_issue_id,
    } = form.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    let existing_issue_id = match newsletter_issue_id.as_deref() {
        None | Some("") => None,
        Some(raw) => Some(Uuid::parse_str(raw).map_err(e400)?),
    };
    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            // Flash must follow the first accept's outcome (stored on the
            // idempotency response), never this request's newsletter_issue_id.
            flash_for_idempotent_replay(&saved_response);
            return Ok(strip_queue_outcome_header(saved_response));
        }
    };
    let issue_id = existing_issue_id.unwrap_or_else(Uuid::new_v4);
    save_newsletter_issue(
        &mut transaction,
        issue_id,
        &title,
        &text_content,
        &html_content,
    )
    .await
    .context("Failed to store newsletter issue details")
    .map_err(e500)?;
    let outcome = queue_issue_for_delivery(&mut transaction, issue_id)
        .await
        .context("Failed to queue newsletter issue for delivery")
        .map_err(e500)?;
    let response = attach_queue_outcome(see_other("/admin/newsletters"), &outcome);
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;
    match outcome {
        QueueOutcome::Queued => success_message().send(),
        QueueOutcome::AlreadyHandled { .. } => already_handled_message().send(),
    }
    Ok(strip_queue_outcome_header(response))
}

fn attach_queue_outcome(mut response: HttpResponse, outcome: &QueueOutcome) -> HttpResponse {
    let value = match outcome {
        QueueOutcome::Queued => "queued",
        QueueOutcome::AlreadyHandled { .. } => "already-handled",
    };
    response.headers_mut().insert(
        HeaderName::from_static(QUEUE_OUTCOME_HEADER),
        HeaderValue::from_static(value),
    );
    response
}

fn strip_queue_outcome_header(mut response: HttpResponse) -> HttpResponse {
    response.headers_mut().remove(QUEUE_OUTCOME_HEADER);
    response
}

/// Replay flash comes from the queue outcome recorded on the first saved
/// response (`x-newsletter-queue-outcome`), which is part of the idempotency
/// headers/body metadata. Looking up `newsletter_issue_id` from the replay
/// body is wrong (body may differ), and re-reading issue status cannot
/// reconstruct Queued vs AlreadyHandled after a successful accept (status is
/// already non-draft). Missing/unknown header ⇒ already-handled (absence is
/// not success — same rule as a missing issue row).
fn flash_for_idempotent_replay(saved_response: &HttpResponse) {
    let outcome = saved_response
        .headers()
        .get(QUEUE_OUTCOME_HEADER)
        .and_then(|v| v.to_str().ok());
    match outcome {
        Some("queued") => success_message().send(),
        // "already-handled", missing, or anything unexpected
        _ => already_handled_message().send(),
    }
}

/// Insert the issue as a `draft` under `newsletter_issue_id`, or, if that id
/// already exists, overwrite its content — but only while it is still a
/// `draft`. Content of a `queued` / `sent` / `failed` issue is never touched:
/// the worker re-reads it for every subscriber, so editing mid-delivery would
/// send different content to different subscribers.
#[tracing::instrument(skip_all)]
async fn save_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, $2, $3, $4, now(), 'draft')
        ON CONFLICT (newsletter_issue_id) DO UPDATE
        SET
            title = EXCLUDED.title,
            text_content = EXCLUDED.text_content,
            html_content = EXCLUDED.html_content
        WHERE newsletter.newsletter_issues.status = 'draft'
        "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    );
    transaction.execute(query).await?;
    Ok(())
}

/// Accept a draft issue for delivery: `draft` → `queued` (stamping `published_at`)
/// and enqueue subscriber tasks in the same transaction. Non-draft statuses are left untouched (no second blast).
/// If there are no confirmed subscribers, flip `queued` → `sent` immediately so the
/// issue does not remain stuck with an empty delivery queue.
#[tracing::instrument(skip_all)]
pub async fn queue_issue_for_delivery(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<QueueOutcome, sqlx::Error> {
    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        FOR UPDATE
        "#,
        newsletter_issue_id
    )
    .fetch_one(&mut **transaction)
    .await?;

    match status {
        NewsletterIssueStatus::Draft => {
            sqlx::query!(
                r#"
                UPDATE newsletter.newsletter_issues
                SET status = 'queued', published_at = now()
                WHERE newsletter_issue_id = $1
                  AND status = 'draft'
                "#,
                newsletter_issue_id
            )
            .execute(&mut **transaction)
            .await?;
            enqueue_delivery_tasks(transaction, newsletter_issue_id).await?;
            // Issue row already held FOR UPDATE above; helper re-takes it (no-op)
            // and shares the emptiness predicate with the worker's delete_task.
            mark_sent_if_drained(transaction, newsletter_issue_id).await?;
            Ok(QueueOutcome::Queued)
        }
        NewsletterIssueStatus::Queued
        | NewsletterIssueStatus::Sent
        | NewsletterIssueStatus::Failed => Ok(QueueOutcome::AlreadyHandled { status }),
    }
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_tasks(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO newsletter.issue_delivery_queue (
            newsletter_issue_id,
            subscriber_email
        )
        SELECT $1, email
        FROM newsletter.subscriptions
        WHERE status = 'confirmed'
        "#,
        newsletter_issue_id,
    );
    transaction.execute(query).await?;
    Ok(())
}
