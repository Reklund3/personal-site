use crate::{configuration::Settings, startup::get_pg_pool};
use crate::{domain::UserEmail, email_client::EmailClient};
use sqlx::{PgPool, Postgres, Transaction};
use std::time::Duration;
use tracing::{Span, field::display};
use uuid::Uuid;

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_pg_pool(&configuration.database);
    let email_client = configuration.email_client.client();
    worker_loop(connection_pool, email_client).await
}

async fn worker_loop(pool: PgPool, email_client: EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&pool, &email_client).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
        }
    }
}

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(
skip_all,
fields(
newsletter_issue_id=tracing::field::Empty,
subscriber_email=tracing::field::Empty
),
err
)]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let task = dequeue_task(pool).await?;
    if task.is_none() {
        return Ok(ExecutionOutcome::EmptyQueue);
    }
    let (transaction, issue_id, email) = task.unwrap();
    Span::current()
        .record("newsletter_issue_id", display(issue_id))
        .record("subscriber_email", display(&email));

    // Enqueue only sets status to `queued`. We flip to `sent` once delivery work
    // for the issue is fully done (queue empty). Permanent failure (`failed`):
    // the issue row is missing when a queue task runs — remaining queue rows for
    // that issue are dropped. Transient DB errors from get_issue propagate so
    // the worker_loop retries. Per-subscriber email API errors are logged and
    // skipped (task removed); they do not mark the issue `failed`.
    // Transient send retries / backoff are tracked in issue #32.
    match get_issue(pool, issue_id).await? {
        Some(issue) => {
            match UserEmail::parse(email.clone()) {
                Ok(email) => {
                    if let Err(e) = email_client
                        .send_email(
                            &email,
                            &issue.title,
                            &issue.html_content,
                            &issue.text_content,
                        )
                        .await
                    {
                        tracing::error!(
                            error.cause_chain = ?e,
                            error.message = %e,
                            "Failed to deliver issue to a confirmed subscriber. \
                                Skipping.",
                        );
                    }
                }
                Err(e) => {
                    tracing::error!(
                        error.cause_chain = ?e,
                        error.message = %e,
                        "Skipping a confirmed subscriber. \
                            Their stored contact details are invalid",
                    );
                }
            }
            delete_task(transaction, issue_id, &email).await?;
            Ok(ExecutionOutcome::TaskCompleted)
        }
        None => {
            tracing::error!(
                %issue_id,
                "Newsletter issue row missing; marking issue failed \
                 and draining its delivery queue.",
            );
            mark_issue_failed_and_drain(transaction, issue_id).await?;
            Ok(ExecutionOutcome::TaskCompleted)
        }
    }
}

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(skip_all)]
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String)>, anyhow::Error> {
    let mut transaction = pool.begin().await?;
    let r = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, subscriber_email
        FROM newsletter.issue_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
        "#,
    )
    .fetch_optional(&mut *transaction)
    .await?;
    if let Some(r) = r {
        Ok(Some((
            transaction,
            r.newsletter_issue_id,
            r.subscriber_email,
        )))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
async fn delete_task(
    mut transaction: PgTransaction,
    issue_id: Uuid,
    email: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        DELETE FROM newsletter.issue_delivery_queue
        WHERE
            newsletter_issue_id = $1 AND
            subscriber_email = $2
        "#,
        issue_id,
        email
    )
    .execute(&mut *transaction)
    .await?;
    // Serialize concurrent workers on the issue row before the emptiness check
    // so two last-row DELETEs cannot both observe NOT EXISTS as false.
    sqlx::query!(
        r#"
        SELECT newsletter_issue_id
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        FOR UPDATE
        "#,
        issue_id
    )
    .fetch_optional(&mut *transaction)
    .await?;
    // When the last queue row for a queued issue is gone, delivery is complete.
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
        issue_id
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(())
}

/// Permanent-failure path: issue content cannot be loaded. Mark `failed` and
/// drop any remaining delivery rows so the worker does not spin forever.
#[tracing::instrument(skip_all)]
async fn mark_issue_failed_and_drain(
    mut transaction: PgTransaction,
    issue_id: Uuid,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        UPDATE newsletter.newsletter_issues
        SET status = 'failed'
        WHERE newsletter_issue_id = $1
          AND status = 'queued'
        "#,
        issue_id,
    )
    .execute(&mut *transaction)
    .await?;
    sqlx::query!(
        r#"
        DELETE FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(())
}

struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

#[tracing::instrument(skip_all)]
async fn get_issue(
    pool: &PgPool,
    issue_id: Uuid,
) -> Result<Option<NewsletterIssue>, anyhow::Error> {
    let issue = sqlx::query_as!(
        NewsletterIssue,
        r#"
        SELECT title, text_content, html_content
        FROM newsletter.newsletter_issues
        WHERE
            newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(issue)
}
