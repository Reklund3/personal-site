use crate::authentication::UserId;
use crate::domain::NewsletterIssueStatus;
use crate::idempotency::{IdempotencyKey, NextAction, save_response, try_processing};
use crate::utils::e400;
use crate::utils::{e500, see_other};
use actix_web::{HttpResponse, web};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
    idempotency_key: String,
    /// Optional existing draft to accept. When set, content fields are ignored
    /// and we only attempt `draft` → `queued` + enqueue (no draft-editing UI yet).
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
            // Re-derive flash from issue status on idempotent replay when the
            // form targeted an existing issue; new publishes always completed
            // as a successful accept on first processing.
            flash_for_idempotent_replay(&pool, existing_issue_id).await;
            return Ok(saved_response);
        }
    };
    let issue_id = match existing_issue_id {
        Some(id) => id,
        None => insert_newsletter_issue(&mut transaction, &title, &text_content, &html_content)
            .await
            .context("Failed to store newsletter issue details")
            .map_err(e500)?,
    };
    let outcome = queue_issue_for_delivery(&mut transaction, issue_id)
        .await
        .context("Failed to queue newsletter issue for delivery")
        .map_err(e500)?;
    let response = see_other("/admin/newsletters");
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;
    match outcome {
        QueueOutcome::Queued => success_message().send(),
        QueueOutcome::AlreadyHandled { .. } => already_handled_message().send(),
    }
    Ok(response)
}

/// Prefer already-handled when replaying an accept of an existing issue that is
/// no longer a draft; otherwise keep the historical success flash for new
/// publishes (and rare draft-still-draft replays).
async fn flash_for_idempotent_replay(pool: &PgPool, existing_issue_id: Option<Uuid>) {
    let Some(issue_id) = existing_issue_id else {
        success_message().send();
        return;
    };
    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();
    match status {
        Some(NewsletterIssueStatus::Draft) | None => success_message().send(),
        Some(_) => already_handled_message().send(),
    }
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<Uuid, sqlx::Error> {
    let newsletter_issue_id = Uuid::new_v4();
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
        "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    );
    transaction.execute(query).await?;
    Ok(newsletter_issue_id)
}

/// Accept a draft issue for delivery: `draft` → `queued` and enqueue subscriber tasks
/// in the same transaction. Non-draft statuses are left untouched (no second blast).
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
    .fetch_optional(&mut **transaction)
    .await?;

    let Some(status) = status else {
        return Err(sqlx::Error::RowNotFound);
    };

    match status {
        NewsletterIssueStatus::Draft => {
            sqlx::query!(
                r#"
                UPDATE newsletter.newsletter_issues
                SET status = 'queued'
                WHERE newsletter_issue_id = $1
                  AND status = 'draft'
                "#,
                newsletter_issue_id
            )
            .execute(&mut **transaction)
            .await?;
            enqueue_delivery_tasks(transaction, newsletter_issue_id).await?;
            // Same NOT EXISTS predicate as delete_task: empty queue ⇒ sent.
            sqlx::query!(
                r#"
                UPDATE newsletter.newsletter_issues
                SET status = 'sent'
                WHERE newsletter_issue_id = $1
                  AND status = 'queued'
                  AND NOT EXISTS (
                      SELECT 1
                      FROM newsletter.issue_delivery_queue
                      WHERE newsletter_issue_id = $1
                  )
                "#,
                newsletter_issue_id
            )
            .execute(&mut **transaction)
            .await?;
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
