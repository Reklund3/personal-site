use crate::authentication::UserId;
use crate::domain::NewsletterIssueStatus;
use crate::idempotency::{IdempotencyKey, NextAction, save_response, try_processing};
use crate::issue_delivery_worker::mark_sent_if_drained;
use crate::utils::e400;
use crate::utils::{e500, see_other};
use actix_web::HttpResponse;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::web;
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
    /// issue is inserted under that id **only if it does not already exist**;
    /// then it is queued. An existing id is refused (redirect to edit) — compose
    /// Publish never overwrites a saved draft.
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

fn already_exists_message() -> FlashMessage {
    FlashMessage::error(
        "This newsletter issue already exists. Open the edit page to publish \
        the saved draft, or view its status. Compose Publish never overwrites \
        an existing issue.",
    )
}

#[derive(Debug, PartialEq, Eq)]
pub enum QueueOutcome {
    /// Transitioned `draft` → `queued` and enqueued delivery rows.
    Queued,
    /// Issue exists but is already `queued` / `sent` / `failed` — do not enqueue again.
    AlreadyHandled { status: NewsletterIssueStatus },
    /// Compose Publish hit an existing `newsletter_issue_id` (insert-only conflict).
    AlreadyExists,
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
    // Replay must run before any insert/exists check so a saved success (or
    // refusal) is returned even when the issue row now exists.
    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            flash_for_idempotent_replay(&saved_response);
            return Ok(strip_queue_outcome_header(saved_response));
        }
    };
    let issue_id = existing_issue_id.unwrap_or_else(Uuid::new_v4);
    let inserted = insert_newsletter_issue(
        &mut transaction,
        issue_id,
        &title,
        &text_content,
        &html_content,
    )
    .await
    .context("Failed to store newsletter issue details")
    .map_err(e500)?;
    if !inserted {
        // Id already taken (e.g. Save draft ran first, or a prior publish).
        // Do not overwrite; send the admin to the edit page.
        let outcome = QueueOutcome::AlreadyExists;
        let response = attach_queue_outcome(
            see_other(&format!("/admin/newsletters/{issue_id}/edit")),
            &outcome,
        );
        let response = save_response(transaction, &idempotency_key, *user_id, response)
            .await
            .map_err(e500)?;
        already_exists_message().send();
        return Ok(strip_queue_outcome_header(response));
    }
    let outcome = queue_issue_for_delivery(&mut transaction, issue_id)
        .await
        .context("Failed to enqueue newsletter issue for delivery")
        .map_err(e500)?;
    let response = attach_queue_outcome(see_other("/admin/newsletters"), &outcome);
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;
    match outcome {
        QueueOutcome::Queued => success_message().send(),
        QueueOutcome::AlreadyHandled { .. } => already_handled_message().send(),
        // Insert just succeeded; queue of a fresh draft cannot be AlreadyExists.
        QueueOutcome::AlreadyExists => unreachable!("fresh insert cannot AlreadyExists"),
    }
    Ok(strip_queue_outcome_header(response))
}

#[derive(serde::Deserialize)]
pub struct PublishSavedPath {
    newsletter_issue_id: Uuid,
}

#[derive(serde::Deserialize)]
pub struct PublishSavedFormData {
    idempotency_key: String,
}

/// Publish a previously saved draft by id: queue the **stored** content under
/// `FOR UPDATE`. No title/body in the request — the edit page cannot send stale
/// form content. Non-draft ⇒ AlreadyHandled; missing id ⇒ 404 (idempotency txn
/// rolled back so the key is not burned).
#[tracing::instrument(
    name = "Publish saved newsletter draft",
    skip_all,
    fields(user_id=%&*user_id)
)]
pub async fn publish_saved_newsletter(
    path: web::Path<PublishSavedPath>,
    form: web::Form<PublishSavedFormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    let issue_id = path.newsletter_issue_id;
    let idempotency_key: IdempotencyKey = form.0.idempotency_key.try_into().map_err(e400)?;

    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::StartProcessing(t) => t,
        NextAction::ReturnSavedResponse(saved_response) => {
            flash_for_idempotent_replay(&saved_response);
            return Ok(strip_queue_outcome_header(saved_response));
        }
    };

    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        FOR UPDATE
        "#,
        issue_id
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(e500)?;

    let Some(status) = status else {
        // Missing id: roll back so the forever key is not consumed on a typo.
        transaction
            .rollback()
            .await
            .context("Failed to rollback publish-saved transaction")
            .map_err(e500)?;
        return Ok(HttpResponse::NotFound()
            .content_type(actix_web::http::header::ContentType::html())
            .body("Newsletter issue not found."));
    };

    let outcome = match status {
        NewsletterIssueStatus::Draft => queue_issue_for_delivery(&mut transaction, issue_id)
            .await
            .context("Failed to enqueue newsletter issue for delivery")
            .map_err(e500)?,
        NewsletterIssueStatus::Queued
        | NewsletterIssueStatus::Sent
        | NewsletterIssueStatus::Failed => QueueOutcome::AlreadyHandled { status },
    };

    let response = attach_queue_outcome(see_other("/admin/newsletters"), &outcome);
    let response = save_response(transaction, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;
    match outcome {
        QueueOutcome::Queued => success_message().send(),
        QueueOutcome::AlreadyHandled { .. } => already_handled_message().send(),
        QueueOutcome::AlreadyExists => unreachable!("publish-saved never AlreadyExists"),
    }
    Ok(strip_queue_outcome_header(response))
}

fn attach_queue_outcome(mut response: HttpResponse, outcome: &QueueOutcome) -> HttpResponse {
    let value = match outcome {
        QueueOutcome::Queued => "queued",
        QueueOutcome::AlreadyHandled { .. } => "already-handled",
        QueueOutcome::AlreadyExists => "already-exists",
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
        Some("already-exists") => already_exists_message().send(),
        // "already-handled", missing, or anything unexpected
        _ => already_handled_message().send(),
    }
}

#[derive(serde::Deserialize)]
pub struct DraftFormData {
    title: String,
    text_content: String,
    html_content: String,
    newsletter_issue_id: String,
    /// Present on the shared form; ignored — draft upsert needs no idempotency row.
    #[serde(default, rename = "idempotency_key")]
    _idempotency_key: Option<String>,
}

/// Persist (or update) a draft without enqueueing delivery. Naturally idempotent via
/// upsert-on-draft; saving over a non-draft issue changes nothing and flashes an error.
#[tracing::instrument(
    name = "Save newsletter draft",
    skip_all,
    fields(user_id=%&*user_id)
)]
pub async fn save_newsletter_draft(
    form: web::Form<DraftFormData>,
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let _user_id = user_id.into_inner();
    let DraftFormData {
        title,
        text_content,
        html_content,
        newsletter_issue_id,
        _idempotency_key: _,
    } = form.0;
    let issue_id = Uuid::parse_str(&newsletter_issue_id).map_err(e400)?;

    let mut transaction = pool
        .begin()
        .await
        .context("Failed to begin draft-save transaction")
        .map_err(e500)?;

    let existing = sqlx::query_scalar!(
        r#"
        SELECT status as "status: NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        FOR UPDATE
        "#,
        issue_id
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(e500)?;

    if let Some(status) = existing
        && status != NewsletterIssueStatus::Draft
    {
        // Drop the txn without writing; content stays untouched.
        transaction
            .rollback()
            .await
            .context("Failed to rollback draft-save transaction")
            .map_err(e500)?;
        FlashMessage::error(
            "This newsletter issue is no longer a draft and cannot be overwritten.",
        )
        .send();
        return Ok(see_other(&format!("/admin/newsletters/{issue_id}/edit")));
    }

    save_newsletter_issue(
        &mut transaction,
        issue_id,
        &title,
        &text_content,
        &html_content,
    )
    .await
    .context("Failed to store newsletter draft")
    .map_err(e500)?;

    // Re-read under the same txn. Covers the race where FOR UPDATE saw no row,
    // then a concurrent publish-by-id inserted+queued before our upsert: ON CONFLICT
    // ... WHERE status = 'draft' writes nothing, but we must not flash success.
    // (Compose Publish is insert-only and would refuse rather than queue an
    // existing id; publish-saved is the path that can race draft save.)
    let status_after = sqlx::query_scalar!(
        r#"
        SELECT status as "status: NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(e500)?;

    if status_after != Some(NewsletterIssueStatus::Draft) {
        transaction
            .rollback()
            .await
            .context("Failed to rollback draft-save after lost race")
            .map_err(e500)?;
        FlashMessage::error(
            "This newsletter issue is no longer a draft and cannot be overwritten.",
        )
        .send();
        return Ok(see_other(&format!("/admin/newsletters/{issue_id}/edit")));
    }

    transaction
        .commit()
        .await
        .context("Failed to commit newsletter draft")
        .map_err(e500)?;

    FlashMessage::info("Draft saved.").send();
    Ok(see_other(&format!("/admin/newsletters/{issue_id}/edit")))
}

/// Insert-only for compose Publish. Returns `true` if a new row was inserted,
/// `false` if `newsletter_issue_id` already existed (`ON CONFLICT DO NOTHING`).
#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: Uuid,
    title: &str,
    text_content: &str,
    html_content: &str,
) -> Result<bool, sqlx::Error> {
    // Bind via `Executor::execute` (same shape as `save_newsletter_issue` /
    // idempotency inserts) so a failed `query!` expansion (type `!`, e.g. when
    // the SARIF clippy job has DATABASE_URL but no Postgres) coerces as an
    // argument instead of having `.rows_affected()` called on `!`.
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
        ON CONFLICT (newsletter_issue_id) DO NOTHING
        "#,
        newsletter_issue_id,
        title,
        text_content,
        html_content
    );
    let result = transaction.execute(query).await?;
    Ok(result.rows_affected() > 0)
}

/// Insert the issue as a `draft` under `newsletter_issue_id`, or, if that id
/// already exists, overwrite its content — but only while it is still a
/// `draft`. Content of a `queued` / `sent` / `failed` issue is never touched:
/// the worker re-reads it for every subscriber, so editing mid-delivery would
/// send different content to different subscribers.
#[tracing::instrument(skip_all)]
pub(crate) async fn save_newsletter_issue(
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
