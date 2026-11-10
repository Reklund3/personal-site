use crate::helpers::{
    ConfirmationLinks, TestApp, assert_is_redirect_to, extract_hidden_value,
    extract_hidden_value_in_form, extract_loaded_hash, spawn_app,
};
use fake::Fake;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use std::time::Duration;
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

async fn create_unconfirmed_subscriber(test_app: &TestApp) -> ConfirmationLinks {
    // We are working with multiple subscribers now,
    // their details must be randomised to avoid conflicts!
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(&serde_json::json!({
        "name": name,
        "email": email
    }))
    .unwrap();

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&test_app.email_server)
        .await;
    test_app
        .post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();
    let email_request = &test_app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    test_app.get_confirmation_links(&email_request)
}

async fn create_confirmed_subscriber(test_app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(test_app).await.html;
    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let test_app = spawn_app().await;
    create_unconfirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&test_app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ));
    test_app.dispatch_all_pending_emails().await;

    // Zero confirmed subscribers ⇒ enqueue inserts nothing ⇒ status flips to sent
    // in the publish transaction (must not remain stuck in `queued`).
    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        ORDER BY published_at DESC
        LIMIT 1
        "#
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .expect("issue row");
    assert_eq!(status, site::domain::NewsletterIssueStatus::Sent);
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ));
    test_app.dispatch_all_pending_emails().await;
    // Mock verifies on Drop that we have sent the newsletter email
}

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_newsletter_form() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app.get_publish_newsletter().await;

    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_publish_a_newsletter() {
    let test_app = spawn_app().await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    // Arrange
    let test_app: TestApp = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act - Part 1 - Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        // We expect the idempotency key as part of the
        // form data, not as an header
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 2 - Follow the redirect
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ));

    // Act - Part 3 - Submit newsletter form **again**
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 4 - Follow the redirect
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ));

    test_app.dispatch_all_pending_emails().await;
    // Mock verifies on Drop that we have sent the newsletter email **once**
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    // Arrange
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        // Setting a long delay to ensure that the second request
        // arrives before the first one completes
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Submit two newsletter forms concurrently
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response1 = app.post_publish_newsletter(&newsletter_request_body);
    let response2 = app.post_publish_newsletter(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);

    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
    app.dispatch_all_pending_emails().await;
    // Mock verifies on Drop that we have sent the newsletter email **once**
}

#[tokio::test]
async fn publish_sets_issue_status_to_queued_then_worker_marks_sent() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Status transition issue",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let row = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        ORDER BY published_at DESC
        LIMIT 1
        "#
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .expect("issue row");
    assert_eq!(row.status, site::domain::NewsletterIssueStatus::Queued);

    let queued = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        row.newsletter_issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(1));

    test_app.dispatch_all_pending_emails().await;

    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        row.newsletter_issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(status, site::domain::NewsletterIssueStatus::Sent);

    let remaining = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        row.newsletter_issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(remaining, Some(0));
}

#[tokio::test]
async fn already_queued_issue_is_not_enqueued_twice() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Draft title', 'text', '<p>html</p>', now(), 'draft')
        "#,
        issue_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let mut tx = test_app.pg_pool.begin().await.unwrap();
    let first = site::routes::queue_issue_for_delivery(&mut tx, issue_id)
        .await
        .unwrap();
    assert_eq!(first, site::routes::QueueOutcome::Queued);
    tx.commit().await.unwrap();

    let count_after_first = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(count_after_first, Some(1));

    let mut tx = test_app.pg_pool.begin().await.unwrap();
    let second = site::routes::queue_issue_for_delivery(&mut tx, issue_id)
        .await
        .unwrap();
    assert!(matches!(
        second,
        site::routes::QueueOutcome::AlreadyHandled {
            status: site::domain::NewsletterIssueStatus::Queued
        }
    ));
    tx.commit().await.unwrap();

    let count_after_second = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(count_after_second, Some(1));
}

#[tokio::test]
async fn publish_of_already_handled_issue_flashes_and_does_not_enqueue() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Already sent', 'text', '<p>html</p>', now(), 'sent')
        "#,
        issue_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&test_app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "idempotency_key": uuid::Uuid::now_v7().to_string(),
    });
    let response = test_app
        .post_publish_saved_newsletter(issue_id, &newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("already been handled"),
        "expected already-handled flash, got: {html_page}"
    );

    let queued = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(0));
}

#[tokio::test]
async fn idempotent_replay_of_already_handled_issue_keeps_already_handled_flash() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Already sent', 'text', '<p>html</p>', now(), 'sent')
        "#,
        issue_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let newsletter_request_body = serde_json::json!({
        "idempotency_key": uuid::Uuid::now_v7().to_string(),
    });

    let response = test_app
        .post_publish_saved_newsletter(issue_id, &newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains("already been handled"));

    // Replay with the same idempotency key must not flash the success message.
    let response = test_app
        .post_publish_saved_newsletter(issue_id, &newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("already been handled"),
        "expected already-handled flash on idempotent replay, got: {html_page}"
    );
    assert!(
        !html_page.contains("emails will go out shortly"),
        "success flash must not appear on AlreadyHandled replay"
    );
}

/// Decodes `%XX` escapes (the flash cookie value is percent-encoded JSON).
fn percent_decode_lossy(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let Some(hex) = raw.get(i + 1..i + 3)
            && let Ok(b) = u8::from_str_radix(hex, 16)
        {
            out.push(b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// R9 refusal contract for a request whose idempotency key was first used for a
/// different request: the response redirects to the list page, the flash it sets
/// has error severity, and the list page renders a flash paragraph that is none
/// of `success_texts`. Every flash renders as `<p><i>..</i></p>` regardless of
/// level, and the list page uses the same markup for its empty-state notes, so
/// the severity is read from the `_flash` cookie on the redirect itself.
/// Returns the list page HTML (the flash is consumed by this read).
async fn assert_refused_to_list_with_error_flash(
    test_app: &TestApp,
    response: reqwest::Response,
    success_texts: &[&str],
) -> String {
    assert_is_redirect_to(&response, "/admin/newsletters");
    let flash_cookies: Vec<String> = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .filter(|v| v.starts_with("_flash="))
        .map(percent_decode_lossy)
        .collect();
    assert!(
        flash_cookies
            .iter()
            .any(|c| c.contains(r#""level":"Error""#)),
        "refusal must set an error-level flash, got cookies: {flash_cookies:?}"
    );

    let html = test_app.get_publish_newsletter_html().await;
    let mut flashes = Vec::new();
    let mut rest = html.as_str();
    while let Some(start) = rest.find("<p><i>") {
        rest = &rest[start + "<p><i>".len()..];
        let Some(end) = rest.find("</i></p>") else {
            break;
        };
        flashes.push(&rest[..end]);
        rest = &rest[end..];
    }
    flashes.retain(|f| *f != "No drafts." && *f != "No published issues yet.");
    assert!(
        !flashes.is_empty(),
        "list page must render the refusal flash: {html}"
    );
    for text in success_texts {
        assert!(
            !html.contains(text),
            "refused request must not flash {text:?}: {html}"
        );
    }
    html
}

async fn delivery_rows_for(test_app: &TestApp, issue_id: uuid::Uuid) -> i64 {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM newsletter.issue_delivery_queue WHERE newsletter_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    n
}

async fn idempotency_rows_for(test_app: &TestApp, key: &str, action: &str) -> i64 {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM idempotency WHERE idempotency_key = $1 AND action = $2",
    )
    .bind(key)
    .bind(action)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    n
}

const SUCCESS_FLASH: &str = "emails will go out shortly";

#[tokio::test]
async fn idempotent_replay_flash_follows_first_outcome_not_replay_body() {
    // L2: replay body may omit/change newsletter_issue_id; flash must still
    // match the first processing outcome stored on the idempotency record.
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    // --- Case A: first accept AlreadyHandled; replay without issue id ---
    let sent_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Already sent', 'text', '<p>html</p>', now(), 'sent')
        "#,
        sent_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let key_a = uuid::Uuid::now_v7().to_string();
    let first_a = serde_json::json!({
        "idempotency_key": key_a,
    });
    let response = test_app
        .post_publish_saved_newsletter(sent_id, &first_a)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains("already been handled"));

    // Replay with the same key (body is only the key for publish-saved).
    let replay_a = serde_json::json!({
        "idempotency_key": key_a,
    });
    let response = test_app
        .post_publish_saved_newsletter(sent_id, &replay_a)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("already been handled"),
        "replay flash must match first AlreadyHandled outcome, got: {html_page}"
    );
    assert!(!html_page.contains("emails will go out shortly"));

    // --- Case B: first new publish Queued; the same key replayed with an
    // unrelated issue id is a different request, so it is refused ---
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let key_b = uuid::Uuid::now_v7().to_string();
    let first_b = serde_json::json!({
        "title": "Fresh issue",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": key_b
    });
    let response = test_app.post_publish_newsletter(&first_b).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains("emails will go out shortly"));

    let unrelated_sent = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Already sent', 'text', '<p>html</p>', now(), 'sent')
        "#,
        unrelated_sent
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let replay_b = serde_json::json!({
        "title": "Fresh issue",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": key_b,
        "newsletter_issue_id": unrelated_sent.to_string()
    });
    let response = test_app.post_publish_newsletter(&replay_b).await;
    assert_refused_to_list_with_error_flash(&test_app, response, &[SUCCESS_FLASH]).await;
    assert_eq!(
        delivery_rows_for(&test_app, unrelated_sent).await,
        0,
        "refused replay must not enqueue delivery for the unrelated issue"
    );
    assert_eq!(idempotency_rows_for(&test_app, &key_b, "publish").await, 1);

    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn email_5xx_does_not_mark_issue_failed() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Will fail to send",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    test_app.dispatch_all_pending_emails().await;

    let row = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        ORDER BY published_at DESC
        LIMIT 1
        "#
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .expect("issue row");
    assert_eq!(
        row.status,
        site::domain::NewsletterIssueStatus::Sent,
        "email API errors skip the subscriber and must not mark the issue failed"
    );
    assert_ne!(row.status, site::domain::NewsletterIssueStatus::Failed);

    let remaining = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        row.newsletter_issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(remaining, Some(0));
}

#[tokio::test]
async fn zero_confirmed_subscribers_marks_issue_sent_on_publish() {
    let test_app = spawn_app().await;
    // No subscribers at all.
    test_app.test_user.login(&test_app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&test_app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Nobody to email",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        ORDER BY published_at DESC
        LIMIT 1
        "#
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(status, site::domain::NewsletterIssueStatus::Sent);

    let queued = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM newsletter.issue_delivery_queue
        "#
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(0));
}

#[tokio::test]
async fn publish_saved_queues_stored_draft_content_unchanged() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Draft to accept', 'draft text', '<p>draft</p>', now() - interval '2 days', 'draft')
        "#,
        issue_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let response = test_app
        .post_publish_saved_newsletter(
            issue_id,
            &serde_json::json!({ "idempotency_key": uuid::Uuid::now_v7().to_string() }),
        )
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains(
        "<p><i>The newsletter issue has been accepted - \
        emails will go out shortly.</i></p>"
    ));

    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(status, site::domain::NewsletterIssueStatus::Queued);

    let queued = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(1));

    // Publish-by-id must queue the STORED draft content, not a request body.
    let issue = sqlx::query!(
        r#"
        SELECT
            title,
            text_content,
            html_content,
            published_at > now() - interval '1 hour' as "published_recently!"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(issue.title, "Draft to accept");
    assert_eq!(issue.text_content, "draft text");
    assert_eq!(issue.html_content, "<p>draft</p>");
    assert!(issue.published_recently);

    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn concurrent_task_completions_flip_issue_to_sent_exactly_once() {
    // Deterministic lock proof for delete_task's FOR UPDATE: hold the issue
    // lock in tx1 via mark_sent_if_drained, assert another connection gets
    // lock_not_available (55P03) on FOR UPDATE NOWAIT, then finish draining.
    let test_app = spawn_app().await;

    let issue_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Racing completions', 'text', '<p>html</p>', now(), 'queued')
        "#,
        issue_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let email_one: String = SafeEmail().fake();
    let email_two: String = SafeEmail().fake();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.issue_delivery_queue (
            newsletter_issue_id,
            subscriber_email
        )
        VALUES ($1, $2), ($1, $3)
        "#,
        issue_id,
        email_one,
        email_two
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let mut tx1 = test_app.pg_pool.begin().await.unwrap();
    sqlx::query!(
        r#"
        DELETE FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1 AND subscriber_email = $2
        "#,
        issue_id,
        email_one
    )
    .execute(&mut *tx1)
    .await
    .unwrap();
    site::issue_delivery_worker::mark_sent_if_drained(&mut tx1, issue_id)
        .await
        .unwrap();

    // Separate connection must not acquire the issue row while tx1 holds it.
    let nowait = sqlx::query_scalar!(
        r#"
        SELECT newsletter_issue_id
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        FOR UPDATE NOWAIT
        "#,
        issue_id
    )
    .fetch_optional(&test_app.pg_pool)
    .await;
    match nowait {
        Err(sqlx::Error::Database(err)) => {
            assert_eq!(
                err.code().as_deref(),
                Some("55P03"),
                "expected lock_not_available, got {:?}",
                err
            );
        }
        other => panic!("expected lock_not_available (55P03), got {other:?}"),
    }

    tx1.commit().await.unwrap();

    let status_after_first = sqlx::query_scalar!(
        r#"
        SELECT status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(
        status_after_first,
        site::domain::NewsletterIssueStatus::Queued,
        "one queue row remains, so status must stay queued"
    );

    let mut tx2 = test_app.pg_pool.begin().await.unwrap();
    sqlx::query!(
        r#"
        DELETE FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1 AND subscriber_email = $2
        "#,
        issue_id,
        email_two
    )
    .execute(&mut *tx2)
    .await
    .unwrap();
    site::issue_delivery_worker::mark_sent_if_drained(&mut tx2, issue_id)
        .await
        .unwrap();
    tx2.commit().await.unwrap();

    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(status, site::domain::NewsletterIssueStatus::Sent);

    let remaining = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(remaining, Some(0));
}

#[tokio::test]
async fn posting_with_an_unused_issue_id_creates_the_issue_under_that_id() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let issue_id = uuid::Uuid::new_v4();
    let newsletter_request_body = serde_json::json!({
        "title": "Brand new issue",
        "text_content": "new text",
        "html_content": "<p>new</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string(),
        "newsletter_issue_id": issue_id.to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let issue = sqlx::query!(
        r#"
        SELECT
            title,
            status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(issue.title, "Brand new issue");
    assert_eq!(issue.status, site::domain::NewsletterIssueStatus::Queued);

    let queued = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(1));

    test_app.dispatch_all_pending_emails().await;
    // Mock verifies on Drop that the issue was delivered once
}

#[tokio::test]
async fn resubmitting_a_queued_issue_id_does_not_change_its_content() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let issue_id = uuid::Uuid::new_v4();
    let first = serde_json::json!({
        "title": "Original title",
        "text_content": "original text",
        "html_content": "<p>original</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string(),
        "newsletter_issue_id": issue_id.to_string()
    });
    let response = test_app.post_publish_newsletter(&first).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Different idempotency key, same issue id, different content: compose
    // Publish is insert-only, so it refuses without overwriting or re-enqueueing.
    let second = serde_json::json!({
        "title": "Changed title",
        "text_content": "changed text",
        "html_content": "<p>changed</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string(),
        "newsletter_issue_id": issue_id.to_string()
    });
    let response = test_app.post_publish_newsletter(&second).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));
    let html_page = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(
        html_page.contains("already exists"),
        "expected already-exists flash, got: {html_page}"
    );

    let title = sqlx::query_scalar!(
        r#"
        SELECT title FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(title, "Original title");

    let queued = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(1));

    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn concurrent_queue_issue_for_delivery_enqueues_exactly_once() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Racing accepts', 'text', '<p>html</p>', now(), 'draft')
        "#,
        issue_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let pool_one = test_app.pg_pool.clone();
    let pool_two = test_app.pg_pool.clone();
    let first = async move {
        let mut tx = pool_one.begin().await.unwrap();
        let outcome = site::routes::queue_issue_for_delivery(&mut tx, issue_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        outcome
    };
    let second = async move {
        let mut tx = pool_two.begin().await.unwrap();
        let outcome = site::routes::queue_issue_for_delivery(&mut tx, issue_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        outcome
    };
    let (first, second) = tokio::join!(first, second);

    let outcomes = [first, second];
    let queued_count = outcomes
        .iter()
        .filter(|o| **o == site::routes::QueueOutcome::Queued)
        .count();
    assert_eq!(queued_count, 1, "exactly one racer must queue the issue");
    let already_handled_count = outcomes
        .iter()
        .filter(|o| {
            matches!(
                o,
                site::routes::QueueOutcome::AlreadyHandled {
                    status: site::domain::NewsletterIssueStatus::Queued
                }
            )
        })
        .count();
    assert_eq!(
        already_handled_count, 1,
        "the other racer must see AlreadyHandled{{ status: Queued }}"
    );

    let queued = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(1));
}

#[tokio::test]
async fn publish_requires_title_and_content_even_with_an_issue_id() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let test_cases = vec![
        (
            serde_json::json!({
                "text_content": "text",
                "html_content": "<p>html</p>",
                "idempotency_key": uuid::Uuid::now_v7().to_string(),
                "newsletter_issue_id": uuid::Uuid::new_v4().to_string()
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Title",
                "html_content": "<p>html</p>",
                "idempotency_key": uuid::Uuid::now_v7().to_string(),
                "newsletter_issue_id": uuid::Uuid::new_v4().to_string()
            }),
            "missing text content",
        ),
        (
            serde_json::json!({
                "title": "Title",
                "text_content": "text",
                "idempotency_key": uuid::Uuid::now_v7().to_string(),
                "newsletter_issue_id": uuid::Uuid::new_v4().to_string()
            }),
            "missing html content",
        ),
    ];

    for (body, error_message) in test_cases {
        let response = test_app.post_publish_newsletter(&body).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn newsletter_issues_published_at_orders_by_time() {
    let test_app = spawn_app().await;

    // published_at is a real timestamp, so ordering is by instant, not by the
    // literal's text. "Earlier instant" is 2026-01-01T10:00Z but its literal
    // sorts lexically after "Later instant" (2026-01-01T12:00Z); text ordering
    // would pick the wrong row. Plain sqlx::query keeps this setup out of the
    // offline cache.
    sqlx::query(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES
            ($1, 'Earlier instant', 'text', '<p>html</p>', '2026-01-02T00:00:00+14:00', 'sent'),
            ($2, 'Later instant', 'text', '<p>html</p>', '2026-01-01T12:00:00+00:00', 'sent')
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .bind(uuid::Uuid::new_v4())
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let latest = sqlx::query_scalar!(
        r#"
        SELECT title
        FROM newsletter.newsletter_issues
        ORDER BY published_at DESC
        LIMIT 1
        "#
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(latest, "Later instant");
}

// Migration 20261006032314 dropped the DEFAULT on newsletter_issues.status so a
// status-unaware writer fails loudly instead of silently becoming a 'draft' (the
// only status publishing will queue for delivery). Both inserts below share every
// other column, so the only difference between them is the missing `status`.
async fn count_newsletter_issues(test_app: &TestApp) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT count(*) FROM newsletter.newsletter_issues")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn inserting_a_newsletter_issue_without_a_status_is_rejected() {
    let test_app = spawn_app().await;
    let count_before = count_newsletter_issues(&test_app).await;

    let error = sqlx::query(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at
        )
        VALUES ($1, 'No status', 'text', '<p>html</p>', now())
        "#,
    )
    .bind(uuid::Uuid::new_v4())
    .execute(&test_app.pg_pool)
    .await
    .expect_err("an insert without a status must not succeed");

    let db_error = error
        .as_database_error()
        .unwrap_or_else(|| panic!("expected a database error, got: {error:?}"));
    // 23502 = not_null_violation
    assert_eq!(db_error.code().as_deref(), Some("23502"));
    assert!(
        db_error.message().contains("\"status\""),
        "the violation should name the status column, got: {}",
        db_error.message()
    );
    assert_eq!(count_newsletter_issues(&test_app).await, count_before);
}

#[tokio::test]
async fn inserting_a_newsletter_issue_with_an_explicit_status_succeeds() {
    // Positive control for the test above: same columns plus `status`.
    let test_app = spawn_app().await;
    let count_before = count_newsletter_issues(&test_app).await;
    let issue_id = uuid::Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'With status', 'text', '<p>html</p>', now(), 'draft')
        "#,
    )
    .bind(issue_id)
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    assert_eq!(count_newsletter_issues(&test_app).await, count_before + 1);
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status::text FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(status, "draft");
}

#[tokio::test]
async fn save_draft_enqueues_nothing_and_leaves_status_draft() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&test_app.email_server)
        .await;

    let issue_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "title": "Draft only",
        "text_content": "draft plain",
        "html_content": "<p>draft html</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app.post_save_newsletter_draft(&body).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));

    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(html.contains("Draft saved."));
    assert!(html.contains("Draft only"));
    assert!(html.contains("draft plain"));

    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(status, site::domain::NewsletterIssueStatus::Draft);

    let queued = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(0));
}

#[tokio::test]
async fn save_edit_then_publish_enqueues_once_with_edited_content() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();

    let save_body = serde_json::json!({
        "title": "First title",
        "text_content": "first plain",
        "html_content": "<p>first</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app.post_save_newsletter_draft(&save_body).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));

    let edit_body = serde_json::json!({
        "title": "Edited title",
        "text_content": "edited plain",
        "html_content": "<p>edited</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app.post_save_newsletter_draft(&edit_body).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));

    // 2b: edit step must persist before any publish.
    let mid = sqlx::query!(
        r#"
        SELECT title, text_content, html_content,
               status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(mid.title, "Edited title");
    assert_eq!(mid.text_content, "edited plain");
    assert_eq!(mid.html_content, "<p>edited</p>");
    assert_eq!(mid.status, site::domain::NewsletterIssueStatus::Draft);
    let edit_html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(edit_html.contains("Edited title"));
    assert!(edit_html.contains("edited plain"));

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let response = test_app
        .post_publish_saved_newsletter(
            issue_id,
            &serde_json::json!({ "idempotency_key": uuid::Uuid::now_v7().to_string() }),
        )
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    test_app.dispatch_all_pending_emails().await;

    // Confirmation email from create_confirmed_subscriber is also on the mock
    // server; find the newsletter send by subject.
    let requests = test_app.email_server.received_requests().await.unwrap();
    let newsletter = requests
        .iter()
        .map(|r| serde_json::from_slice::<serde_json::Value>(&r.body).unwrap())
        .find(|b| b["Subject"].as_str() == Some("Edited title"))
        .expect("newsletter email with edited title");
    assert_eq!(newsletter["TextBody"].as_str().unwrap(), "edited plain");
    assert_eq!(newsletter["HtmlBody"].as_str().unwrap(), "<p>edited</p>");

    let row = sqlx::query!(
        r#"
        SELECT title, text_content, html_content,
               status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(row.title, "Edited title");
    assert_eq!(row.text_content, "edited plain");
    assert_eq!(row.html_content, "<p>edited</p>");
    assert_eq!(row.status, site::domain::NewsletterIssueStatus::Sent);

    let queue_count =
        sqlx::query_scalar!(r#"SELECT COUNT(*) FROM newsletter.issue_delivery_queue"#)
            .fetch_one(&test_app.pg_pool)
            .await
            .unwrap();
    assert_eq!(queue_count, Some(0));
}

#[tokio::test]
async fn save_draft_over_queued_issue_is_rejected_without_changes() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Queued title', 'queued text', '<p>queued</p>', now(), 'queued')
        "#,
        issue_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let body = serde_json::json!({
        "title": "Hijack attempt",
        "text_content": "should not land",
        "html_content": "<p>nope</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app.post_save_newsletter_draft(&body).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));
    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(
        html.contains("no longer a draft"),
        "expected rejection flash, got: {html}"
    );
    assert!(html.contains("Queued title"));
    assert!(!html.contains("Hijack attempt"));

    let row = sqlx::query!(
        r#"
        SELECT title, text_content, html_content,
               status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(row.title, "Queued title");
    assert_eq!(row.text_content, "queued text");
    assert_eq!(row.html_content, "<p>queued</p>");
    assert_eq!(row.status, site::domain::NewsletterIssueStatus::Queued);
}

#[tokio::test]
async fn edit_of_unknown_issue_returns_404() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let response = test_app.get_newsletter_edit(uuid::Uuid::new_v4()).await;
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn draft_routes_require_login() {
    let app = spawn_app().await;
    let issue_id = uuid::Uuid::new_v4();

    let response = app.get_newsletter_new_form().await;
    assert_is_redirect_to(&response, "/login");

    let response = app.get_newsletter_edit(issue_id).await;
    assert_is_redirect_to(&response, "/login");

    let body = serde_json::json!({
        "title": "x",
        "text_content": "y",
        "html_content": "z",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = app.post_save_newsletter_draft(&body).await;
    assert_is_redirect_to(&response, "/login");

    let response = app
        .post_publish_saved_newsletter(
            issue_id,
            &serde_json::json!({ "idempotency_key": uuid::Uuid::now_v7().to_string() }),
        )
        .await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_form_includes_issue_id_and_both_buttons() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let html = test_app
        .get_newsletter_new_form()
        .await
        .text()
        .await
        .unwrap();
    assert!(html.contains(r#"name="newsletter_issue_id""#));
    assert!(html.contains("Save draft"));
    assert!(html.contains("Publish"));
    assert!(html.contains(r#"formaction="/admin/newsletters/drafts""#));
    assert!(html.contains(r#"action="/admin/newsletters""#));
}

#[tokio::test]
async fn edit_form_wires_path_id_and_has_no_content_publish() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    test_app
        .post_save_newsletter_draft(&serde_json::json!({
            "title": "Wire me",
            "text_content": "x",
            "html_content": "<p>x</p>",
            "newsletter_issue_id": issue_id.to_string(),
            "idempotency_key": uuid::Uuid::now_v7().to_string()
        }))
        .await;

    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(
        html.contains(&format!(r#"name="newsletter_issue_id" value="{issue_id}""#)),
        "edit form must reuse path id: {html}"
    );
    assert!(
        html.contains(&format!(
            r#"action="/admin/newsletters/{issue_id}/publish""#
        )),
        "edit page needs publish-by-id form: {html}"
    );
    let publish_action = format!("/admin/newsletters/{issue_id}/publish");
    let key = extract_hidden_value_in_form(&html, &publish_action, "idempotency_key")
        .expect("idempotency key must be in the publish-by-id form");
    let key = uuid::Uuid::parse_str(&key).expect("publish key must be a UUID");
    assert_eq!(key.get_version_num(), 7, "form-minted key must be UUIDv7");
    assert!(!key.is_nil(), "publish key must not be the nil UUID");
    let html_again = test_app.get_newsletter_edit_html(issue_id).await;
    let key_again =
        extract_hidden_value_in_form(&html_again, &publish_action, "idempotency_key").unwrap();
    assert_ne!(
        key.to_string(),
        key_again,
        "each edit page load must mint a fresh publish key"
    );
    // Content form posts only to drafts — never compose Publish with body fields.
    assert!(html.contains(r#"action="/admin/newsletters/drafts""#));
    assert!(
        !html.contains(r#"action="/admin/newsletters""#),
        "edit page must not content-publish to POST /admin/newsletters: {html}"
    );
    assert!(
        !html.contains(r#"formaction="/admin/newsletters""#),
        "edit page must not formaction compose publish"
    );
}

#[tokio::test]
async fn list_page_shows_drafts_with_edit_links() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    let sent_id = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "title": "Listed draft",
        "text_content": "plain",
        "html_content": "<p>x</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    test_app.post_save_newsletter_draft(&body).await;
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id, title, text_content, html_content, published_at, status
        ) VALUES ($1, 'Listed sent', 't', '<p>t</p>', now(), 'sent')
        "#,
        sent_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains("New issue"));
    let drafts_at = html.find("<h2>Drafts</h2>").expect("Drafts heading");
    let recent_at = html.find("<h2>Recent issues</h2>").expect("Recent heading");
    assert!(drafts_at < recent_at);
    let drafts_section = &html[drafts_at..recent_at];
    let recent_section = &html[recent_at..];
    assert!(drafts_section.contains("Listed draft"));
    assert!(drafts_section.contains(&format!("/admin/newsletters/{issue_id}/edit")));
    assert!(drafts_section.contains(&format!(
        r#"action="/admin/newsletters/{issue_id}/publish""#
    )));
    assert!(!drafts_section.contains("Listed sent"));
    assert!(recent_section.contains("Listed sent"));
    assert!(!recent_section.contains("Listed draft"));
}

#[tokio::test]
async fn read_only_issue_view_has_no_publish_form() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id,
            title,
            text_content,
            html_content,
            published_at,
            status
        )
        VALUES ($1, 'Sent title', 'sent text', '<p>sent</p>', now(), 'sent')
        "#,
        issue_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(
        html.contains("read-only"),
        "expected read-only banner: {html}"
    );
    assert!(html.contains("Sent title"));
    assert!(html.contains("sent text"));
    assert!(
        !html.contains("Save draft"),
        "read-only must not offer Save draft"
    );
    assert!(
        !html.contains(">Publish<"),
        "read-only must not offer Publish"
    );
    assert!(
        !html.contains(r#"action="/admin/newsletters""#),
        "read-only must not render a POST form: {html}"
    );
    assert!(
        !html.contains(r#"name="idempotency_key""#),
        "read-only must not mint an idempotency key field"
    );
    assert!(
        !html.contains("/publish"),
        "read-only must not offer publish-by-id"
    );
}

#[tokio::test]
async fn prefilled_and_listed_titles_are_html_escaped() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    // Include & and ' so dropping those escape_attr arms would fail.
    let nasty_title = r#""><script>alert(1)</script> & O'Brien<"#;
    let nasty_text = "</textarea><script>alert(1)</script> & 'x'";
    let nasty_html = r#"<img src=x onerror="alert(1)"> & 'y'"#;
    let body = serde_json::json!({
        "title": nasty_title,
        "text_content": nasty_text,
        "html_content": nasty_html,
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    test_app.post_save_newsletter_draft(&body).await;

    let edit = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(
        edit.contains("&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt; &amp; O&#39;Brien&lt;"),
        "title must escape quotes, angles, amp, apostrophe: {edit}"
    );
    assert!(
        edit.contains("&lt;/textarea&gt;&lt;script&gt;alert(1)&lt;/script&gt; &amp; &#39;x&#39;"),
        "textarea body must escape closers/amp/apostrophe: {edit}"
    );
    assert!(
        !edit.contains("</textarea><script>"),
        "raw textarea breakout must not appear"
    );
    assert!(
        edit.contains("&lt;img src=x onerror=&quot;alert(1)&quot;&gt; &amp; &#39;y&#39;"),
        "html_content must be escaped in the form: {edit}"
    );

    let list = test_app.get_publish_newsletter_html().await;
    let drafts_at = list.find("<h2>Drafts</h2>").expect("Drafts");
    let recent_at = list.find("<h2>Recent issues</h2>").expect("Recent");
    let drafts_section = &list[drafts_at..recent_at];
    assert!(
        drafts_section
            .contains("&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt; &amp; O&#39;Brien&lt;"),
        "draft list titles must be escaped: {drafts_section}"
    );
    assert!(
        !list.contains("<script>alert(1)</script>"),
        "raw script must not appear in list HTML"
    );

    // Recent-issues sink: a non-draft title with the same hostile payload.
    let sent_id = uuid::Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO newsletter.newsletter_issues (
            newsletter_issue_id, title, text_content, html_content, published_at, status
        ) VALUES ($1, $2, 't', '<p>t</p>', now(), 'sent')
        "#,
        sent_id,
        nasty_title
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();
    let list = test_app.get_publish_newsletter_html().await;
    let recent_at = list.find("<h2>Recent issues</h2>").expect("Recent");
    let recent_section = &list[recent_at..];
    assert!(
        recent_section
            .contains("&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt; &amp; O&#39;Brien&lt;"),
        "recent-list titles must be escaped: {recent_section}"
    );
    assert!(
        !recent_section.contains("<script>alert(1)</script>"),
        "raw script must not appear in recent list"
    );

    // Hostile flash sink: trigger AlreadyExists; flash is server-authored and
    // routed through escape_attr. Also force a user-influenced redirect page
    // that embeds the flash HTML and assert no raw angle brackets from a
    // crafted title leak into the flash paragraph via other sinks — the flash
    // text itself is fixed, so assert the <p><i>…</i></p> wrapper is escaped
    // content only (unit tests cover escape_attr for flash-like payloads).
    let stale = serde_json::json!({
        "title": nasty_title,
        "text_content": nasty_text,
        "html_content": nasty_html,
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app.post_publish_newsletter(&stale).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));
    let edit = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(
        edit.contains("already exists"),
        "expected already-exists flash: {edit}"
    );
    // Flash paragraph must not contain a raw <script> even if titles are nasty.
    let flash_start = edit.find("<p><i>").expect("flash");
    let flash_end = edit[flash_start..].find("</i></p>").expect("flash end") + flash_start;
    let flash = &edit[flash_start..=flash_end + "</i></p>".len() - 1];
    assert!(
        !flash.contains("<script>") && !flash.contains("</script>"),
        "flash sink must not embed raw script tags: {flash}"
    );
}

#[tokio::test]
async fn draft_save_writes_idempotency_row_under_save_draft_action() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    let key = uuid::Uuid::now_v7();
    let body = serde_json::json!({
        "title": "Idem row",
        "text_content": "x",
        "html_content": "<p>x</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "loaded_content_hash": "",
        "idempotency_key": key.to_string()
    });
    let response = test_app.post_save_newsletter_draft(&body).await;
    assert_eq!(response.status().as_u16(), 303);
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));

    let draft_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1"#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(draft_count, Some(1));

    let action = sqlx::query_scalar!(
        r#"SELECT action FROM idempotency WHERE idempotency_key = $1"#,
        key.to_string()
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(action, "save_draft");
}

#[tokio::test]
async fn malformed_edit_path_uuid_is_not_a_500() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let response = test_app
        .api_client
        .get(format!(
            "{}/admin/newsletters/not-a-uuid/edit",
            test_app.address
        ))
        .send()
        .await
        .expect("request");
    // The route pattern matches any segment; the `Path<Uuid>` extractor rejects
    // the non-UUID one, and Actix's default path error is a 404 (not a 500).
    assert_eq!(response.status().as_u16(), 404);
    // Prove the route is registered: a valid stored id on the same path is 200.
    let issue_id = uuid::Uuid::new_v4();
    test_app
        .post_save_newsletter_draft(&serde_json::json!({
            "title": "x", "text_content": "y", "html_content": "z",
            "newsletter_issue_id": issue_id.to_string(),
        }))
        .await;
    let response = test_app.get_newsletter_edit(issue_id).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn malformed_newsletter_issue_id_in_draft_post_is_400() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let body = serde_json::json!({
        "title": "x",
        "text_content": "y",
        "html_content": "z",
        "newsletter_issue_id": "not-a-uuid",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app.post_save_newsletter_draft(&body).await;
    assert_eq!(response.status().as_u16(), 400);
}

/// Wait (polling, 10s cap) until `n` backends in this test's database are blocked
/// on a heavyweight lock. Each test has its own database, so the count is scoped.
async fn wait_for_lock_waiters(test_app: &TestApp, n: i64) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let (waiting,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM pg_stat_activity \
             WHERE datname = current_database() AND wait_event_type = 'Lock'",
        )
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
        if waiting >= n {
            return;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {n} lock waiter(s); saw {waiting}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn issue_status_and_queue(
    test_app: &TestApp,
    issue_id: uuid::Uuid,
) -> (String, (String, String, String), i64) {
    let (status,): (String,) = sqlx::query_as(
        "SELECT status::text FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    let (queued,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM newsletter.issue_delivery_queue WHERE newsletter_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    (status, stored_content(test_app, issue_id).await, queued)
}

async fn logged_in_client(test_app: &TestApp) -> reqwest::Client {
    let client = TestApp::fresh_api_client();
    test_app.login_client(&client).await;
    client
}

async fn edit_html_for(test_app: &TestApp, client: &reqwest::Client, id: uuid::Uuid) -> String {
    client
        .get(format!("{}/admin/newsletters/{id}/edit", test_app.address))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap()
}

/// Seed a draft; return the hash an edit tab would carry.
async fn seed_draft_for_race(test_app: &TestApp, issue_id: uuid::Uuid) -> String {
    test_app.test_user.login(test_app).await;
    test_app
        .post_save_newsletter_draft(&serde_json::json!({
            "title": "Seed title", "text_content": "seed plain", "html_content": "<p>seed</p>",
            "newsletter_issue_id": issue_id.to_string(),
        }))
        .await;
    extract_loaded_hash(&test_app.get_newsletter_edit_html(issue_id).await).unwrap()
}

fn spawn_post(
    client: reqwest::Client,
    url: String,
    body: serde_json::Value,
) -> tokio::task::JoinHandle<reqwest::Response> {
    tokio::spawn(async move { client.post(url).form(&body).send().await.expect("post") })
}

async fn lock_issue_row(
    test_app: &TestApp,
    issue_id: uuid::Uuid,
) -> sqlx::Transaction<'static, sqlx::Postgres> {
    let mut gate = test_app.pg_pool.begin().await.unwrap();
    sqlx::query(
        "SELECT 1 FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1 FOR UPDATE",
    )
    .bind(issue_id)
    .execute(&mut *gate)
    .await
    .unwrap();
    gate
}

async fn mount_one_email(test_app: &TestApp) {
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;
}

const NOT_A_DRAFT_FLASH: &str =
    "This newsletter issue is no longer a draft and cannot be overwritten.";

// R2a. Interleavings are forced, not hoped for: the test holds the issue row's
// FOR UPDATE lock in its own transaction, fires request A and waits (via
// pg_stat_activity, no fixed sleeps) until A blocks on it, then fires B and
// waits until B queues behind A, then releases. Postgres grants a contended row
// lock to waiters in arrival order, so A runs to commit before B.

#[tokio::test]
async fn race_save_then_publish_by_id_queues_saved_content() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    mount_one_email(&test_app).await;
    let issue_id = uuid::Uuid::new_v4();
    let hash = seed_draft_for_race(&test_app, issue_id).await;
    let save_client = logged_in_client(&test_app).await;
    let publish_client = logged_in_client(&test_app).await;

    let gate = lock_issue_row(&test_app, issue_id).await;
    let save = spawn_post(
        save_client.clone(),
        format!("{}/admin/newsletters/drafts", test_app.address),
        serde_json::json!({
            "title": "Saved title", "text_content": "saved plain", "html_content": "<p>saved</p>",
            "newsletter_issue_id": issue_id.to_string(), "loaded_content_hash": hash,
            "idempotency_key": uuid::Uuid::now_v7().to_string(),
        }),
    );
    wait_for_lock_waiters(&test_app, 1).await;
    let publish = spawn_post(
        publish_client,
        format!("{}/admin/newsletters/{issue_id}/publish", test_app.address),
        serde_json::json!({ "idempotency_key": uuid::Uuid::now_v7().to_string() }),
    );
    wait_for_lock_waiters(&test_app, 2).await;
    gate.rollback().await.unwrap();

    assert_eq!(save.await.unwrap().status().as_u16(), 303);
    assert_eq!(publish.await.unwrap().status().as_u16(), 303);
    let (status, content, queued) = issue_status_and_queue(&test_app, issue_id).await;
    assert!(status == "queued" || status == "sent", "status {status}");
    assert_eq!(
        content,
        (
            "Saved title".into(),
            "saved plain".into(),
            "<p>saved</p>".into()
        )
    );
    assert_eq!(queued, 1, "exactly one delivery row");
    let html = edit_html_for(&test_app, &save_client, issue_id).await;
    assert!(html.contains("Draft saved."), "save ran first: {html}");
    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn race_publish_by_id_then_save_refuses_save() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    mount_one_email(&test_app).await;
    let issue_id = uuid::Uuid::new_v4();
    let hash = seed_draft_for_race(&test_app, issue_id).await;
    let save_client = logged_in_client(&test_app).await;
    let publish_client = logged_in_client(&test_app).await;

    let gate = lock_issue_row(&test_app, issue_id).await;
    let publish = spawn_post(
        publish_client,
        format!("{}/admin/newsletters/{issue_id}/publish", test_app.address),
        serde_json::json!({ "idempotency_key": uuid::Uuid::now_v7().to_string() }),
    );
    wait_for_lock_waiters(&test_app, 1).await;
    let save = spawn_post(
        save_client.clone(),
        format!("{}/admin/newsletters/drafts", test_app.address),
        serde_json::json!({
            "title": "Late title", "text_content": "late plain", "html_content": "<p>late</p>",
            "newsletter_issue_id": issue_id.to_string(), "loaded_content_hash": hash,
            "idempotency_key": uuid::Uuid::now_v7().to_string(),
        }),
    );
    wait_for_lock_waiters(&test_app, 2).await;
    gate.rollback().await.unwrap();

    assert_eq!(publish.await.unwrap().status().as_u16(), 303);
    assert_eq!(save.await.unwrap().status().as_u16(), 303);
    let (status, content, queued) = issue_status_and_queue(&test_app, issue_id).await;
    assert!(status == "queued" || status == "sent", "status {status}");
    assert_eq!(
        content,
        (
            "Seed title".into(),
            "seed plain".into(),
            "<p>seed</p>".into()
        )
    );
    assert_eq!(queued, 1, "exactly one delivery row");
    let html = edit_html_for(&test_app, &save_client, issue_id).await;
    assert!(
        html.contains(NOT_A_DRAFT_FLASH),
        "save must be refused: {html}"
    );
    assert!(!html.contains("Draft saved."));
    test_app.dispatch_all_pending_emails().await;
}

// R2b. No-row race: a compose-page first save (empty hash, no row yet) loses to
// a compose Publish of the same id. To pin the order the test takes ACCESS
// EXCLUSIVE on `subscriptions`, which compose Publish first reads in
// `queue_issue_for_delivery` only *after* inserting the issue row. So:
//   1. Publish inserts the row (uncommitted) and blocks on `subscriptions`.
//   2. Save's FOR UPDATE sees no row (uncommitted rows are invisible), then its
//      insert-only INSERT ... ON CONFLICT DO NOTHING blocks on Publish's key.
//   3. Release: Publish queues and commits; Save's insert does nothing, the
//      row is now `queued`, and only the post-write status re-check refuses it.
#[tokio::test]
async fn race_compose_publish_inserts_before_first_save_refuses_save() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    mount_one_email(&test_app).await;
    let issue_id = uuid::Uuid::new_v4();
    let save_client = logged_in_client(&test_app).await;
    let publish_client = logged_in_client(&test_app).await;

    let mut gate = test_app.pg_pool.begin().await.unwrap();
    sqlx::query("LOCK TABLE newsletter.subscriptions IN ACCESS EXCLUSIVE MODE")
        .execute(&mut *gate)
        .await
        .unwrap();

    let publish = spawn_post(
        publish_client,
        format!("{}/admin/newsletters", test_app.address),
        serde_json::json!({
            "title": "Published title", "text_content": "pub plain", "html_content": "<p>pub</p>",
            "newsletter_issue_id": issue_id.to_string(),
            "idempotency_key": uuid::Uuid::now_v7().to_string(),
        }),
    );
    wait_for_lock_waiters(&test_app, 1).await;
    let save = spawn_post(
        save_client.clone(),
        format!("{}/admin/newsletters/drafts", test_app.address),
        serde_json::json!({
            "title": "Compose save", "text_content": "save plain", "html_content": "<p>save</p>",
            "newsletter_issue_id": issue_id.to_string(), "loaded_content_hash": "",
            "idempotency_key": uuid::Uuid::now_v7().to_string(),
        }),
    );
    wait_for_lock_waiters(&test_app, 2).await;
    gate.rollback().await.unwrap();

    assert_eq!(publish.await.unwrap().status().as_u16(), 303);
    assert_eq!(save.await.unwrap().status().as_u16(), 303);
    let (status, content, queued) = issue_status_and_queue(&test_app, issue_id).await;
    assert!(status == "queued" || status == "sent", "status {status}");
    assert_eq!(
        content,
        (
            "Published title".into(),
            "pub plain".into(),
            "<p>pub</p>".into()
        )
    );
    assert_eq!(queued, 1, "exactly one delivery row");
    let html = edit_html_for(&test_app, &save_client, issue_id).await;
    assert!(
        html.contains(NOT_A_DRAFT_FLASH),
        "re-check must refuse the lost-race save: {html}"
    );
    assert!(!html.contains("Draft saved."));
    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn compose_publish_after_save_refuses_without_overwrite_or_enqueue() {
    // Finding 1: stale compose Publish cannot send old content once a draft exists.
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    let v1 = serde_json::json!({
        "title": "Version One",
        "text_content": "plain v1",
        "html_content": "<p>v1</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    test_app.post_save_newsletter_draft(&v1).await;

    let v2 = serde_json::json!({
        "title": "Version Two",
        "text_content": "plain v2",
        "html_content": "<p>v2</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    test_app.post_save_newsletter_draft(&v2).await;

    // Scoped: drop before publish-by-id so its delivery email does not match expect(0).
    {
        let _no_email = Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(0)
            .mount_as_scoped(&test_app.email_server)
            .await;

        // Stale tab still holds v1 and hits compose Publish.
        let response = test_app.post_publish_newsletter(&v1).await;
        assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));
        let html = test_app.get_newsletter_edit_html(issue_id).await;
        assert!(
            html.contains("already exists"),
            "expected already-exists flash, got: {html}"
        );

        let row = sqlx::query!(
            r#"
            SELECT title, text_content, status as "status: site::domain::NewsletterIssueStatus"
            FROM newsletter.newsletter_issues
            WHERE newsletter_issue_id = $1
            "#,
            issue_id
        )
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
        assert_eq!(row.title, "Version Two");
        assert_eq!(row.text_content, "plain v2");
        assert_eq!(row.status, site::domain::NewsletterIssueStatus::Draft);

        let queued = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM newsletter.issue_delivery_queue
            WHERE newsletter_issue_id = $1
            "#,
            issue_id
        )
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
        assert_eq!(queued, Some(0));
    }

    // publish-by-id queues the saved DB content (v2).
    create_confirmed_subscriber(&test_app).await;
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;
    let response = test_app
        .post_publish_saved_newsletter(
            issue_id,
            &serde_json::json!({ "idempotency_key": uuid::Uuid::now_v7().to_string() }),
        )
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    test_app.dispatch_all_pending_emails().await;

    let row = sqlx::query!(
        r#"
        SELECT title, text_content,
               status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(row.title, "Version Two");
    assert_eq!(row.text_content, "plain v2");
    assert_eq!(row.status, site::domain::NewsletterIssueStatus::Sent);
}

#[tokio::test]
async fn compose_publish_idempotent_replay_skips_already_exists_check() {
    // A successful compose Publish must replay even though the row now exists.
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let issue_id = uuid::Uuid::new_v4();
    let key = uuid::Uuid::now_v7().to_string();
    let body = serde_json::json!({
        "title": "Once",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": key
    });
    let response = test_app.post_publish_newsletter(&body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains("emails will go out shortly"));

    let response = test_app.post_publish_newsletter(&body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(
        html.contains("emails will go out shortly"),
        "replay must return first Queued flash, not already-exists: {html}"
    );
    assert!(!html.contains("already exists"));

    // Replay must not create a second delivery row.
    let queued = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM newsletter.issue_delivery_queue WHERE newsletter_issue_id = $1"#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(1));
    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn publish_saved_unknown_id_returns_404() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let response = test_app
        .post_publish_saved_newsletter(
            uuid::Uuid::new_v4(),
            &serde_json::json!({ "idempotency_key": uuid::Uuid::now_v7().to_string() }),
        )
        .await;
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn textarea_round_trip_preserves_leading_newline() {
    // HTML parsers drop one LF immediately after <textarea>; emit an extra
    // newline after the opening tag so stored leading newlines survive.
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    let text_content = "\nleading plain";
    let html_content = "\n<p>leading html</p>";
    let body = serde_json::json!({
        "title": "Leading newline",
        "text_content": text_content,
        "html_content": html_content,
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app.post_save_newsletter_draft(&body).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));

    let row = sqlx::query!(
        r#"
        SELECT text_content, html_content
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(row.text_content, text_content);
    assert_eq!(row.html_content, html_content);

    let html = test_app.get_newsletter_edit_html(issue_id).await;
    // After the opening tag: synthetic \n (dropped by parsers) + content's \n.
    assert!(
        html.contains(">\n\nleading plain</textarea>"),
        "server HTML must contain >\\n + content with leading newline: {html}"
    );
    assert!(
        html.contains(">\n\n&lt;p&gt;leading html&lt;/p&gt;</textarea>"),
        "html_content leading newline must survive in textarea: {html}"
    );
}

#[tokio::test]
async fn empty_title_draft_lists_as_untitled() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let issue_id = uuid::Uuid::new_v4();
    test_app
        .post_save_newsletter_draft(&serde_json::json!({
            "title": "",
            "text_content": "body",
            "html_content": "<p>body</p>",
            "newsletter_issue_id": issue_id.to_string(),
            "idempotency_key": uuid::Uuid::now_v7().to_string()
        }))
        .await;
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains("(untitled)"));
    assert!(html.contains(&format!("/admin/newsletters/{issue_id}/edit")));
    assert!(!html.contains(&format!(
        r#"<a href="/admin/newsletters/{issue_id}/edit"></a>"#
    )));
}

async fn stored_content(test_app: &TestApp, issue_id: uuid::Uuid) -> (String, String, String) {
    sqlx::query_as::<_, (String, String, String)>(
        "SELECT title, text_content, html_content FROM newsletter.newsletter_issues \
         WHERE newsletter_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap()
}

const STALE_FLASH: &str = "This draft changed since you opened it";

#[tokio::test]
async fn stale_tab_save_is_refused_and_publish_sends_newer_draft() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    let v1 = serde_json::json!({
        "title": "v1", "text_content": "v1 text", "html_content": "<p>v1</p>",
        "newsletter_issue_id": issue_id.to_string(),
    });
    let r = test_app.post_save_newsletter_draft(&v1).await;
    assert_is_redirect_to(&r, &format!("/admin/newsletters/{issue_id}/edit"));

    // Tab A loads v1.
    let tab_a_html = test_app.get_newsletter_edit_html(issue_id).await;
    let tab_a_hash = extract_loaded_hash(&tab_a_html).expect("edit page carries hash");
    assert_eq!(tab_a_hash.len(), 64);

    // Tab B (fresh load) saves v2.
    let v2 = serde_json::json!({
        "title": "v2", "text_content": "v2 text", "html_content": "<p>v2</p>",
        "newsletter_issue_id": issue_id.to_string(),
    });
    test_app.post_save_newsletter_draft(&v2).await;
    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(html.contains("Draft saved."));
    assert_ne!(extract_loaded_hash(&html).unwrap(), tab_a_hash);

    // Tab A saves v1 with its stale hash: refused, v2 kept.
    // Use the key Tab A minted when it loaded (still valid v7); missing key is 400.
    let tab_a_key = extract_hidden_value(&tab_a_html, "idempotency_key").expect("tab A key");
    let mut stale = v1.clone();
    stale["loaded_content_hash"] = tab_a_hash.into();
    stale["idempotency_key"] = tab_a_key.into();
    let r = test_app.post_save_newsletter_draft_raw(&stale).await;
    assert_is_redirect_to(&r, &format!("/admin/newsletters/{issue_id}/edit"));
    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(html.contains(STALE_FLASH), "stale flash expected: {html}");
    assert!(!html.contains("Draft saved."));
    assert_eq!(
        stored_content(&test_app, issue_id).await,
        ("v2".into(), "v2 text".into(), "<p>v2</p>".into())
    );

    // Tab A reloads (new hash) and publishes by id: v2 is what gets queued.
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;
    let key = extract_hidden_value(&html, "idempotency_key").expect("publish key");
    let r = test_app
        .post_publish_saved_newsletter(issue_id, &serde_json::json!({ "idempotency_key": key }))
        .await;
    assert_is_redirect_to(&r, "/admin/newsletters");
    let (status,): (String,) = sqlx::query_as(
        "SELECT status::text FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert!(status == "queued" || status == "sent", "status {status}");
    assert_eq!(stored_content(&test_app, issue_id).await.0, "v2");
    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn save_reload_save_succeeds_with_fresh_hash() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let issue_id = uuid::Uuid::new_v4();
    for (i, title) in ["first", "second", "third"].iter().enumerate() {
        let hash = if i == 0 {
            String::new()
        } else {
            let html = test_app.get_newsletter_edit_html(issue_id).await;
            extract_loaded_hash(&html).unwrap()
        };
        // Each intentional save needs a fresh key (reload mints a new UUIDv7).
        let r = test_app
            .post_save_newsletter_draft_raw(&serde_json::json!({
                "title": title, "text_content": "t", "html_content": "<p>t</p>",
                "newsletter_issue_id": issue_id.to_string(),
                "loaded_content_hash": hash,
                "idempotency_key": uuid::Uuid::now_v7().to_string(),
            }))
            .await;
        assert_is_redirect_to(&r, &format!("/admin/newsletters/{issue_id}/edit"));
        let html = test_app.get_newsletter_edit_html(issue_id).await;
        assert!(
            html.contains("Draft saved."),
            "save {i} must succeed: {html}"
        );
        assert_eq!(stored_content(&test_app, issue_id).await.0, *title);
    }
}

#[tokio::test]
async fn missing_malformed_or_tampered_hash_is_refused() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let issue_id = uuid::Uuid::new_v4();
    test_app
        .post_save_newsletter_draft(&serde_json::json!({
            "title": "orig", "text_content": "t", "html_content": "h",
            "newsletter_issue_id": issue_id.to_string(),
        }))
        .await;
    let real = extract_loaded_hash(&test_app.get_newsletter_edit_html(issue_id).await).unwrap();
    let body = |hash: Option<String>| {
        let mut b = serde_json::json!({
            "title": "evil", "text_content": "t", "html_content": "h",
            "newsletter_issue_id": issue_id.to_string(),
            "idempotency_key": uuid::Uuid::now_v7().to_string(),
        });
        if let Some(h) = hash {
            b["loaded_content_hash"] = h.into();
        }
        b
    };

    // Missing field and malformed value: 400.
    let r = test_app.post_save_newsletter_draft_raw(&body(None)).await;
    assert_eq!(r.status().as_u16(), 400);
    let r = test_app
        .post_save_newsletter_draft_raw(&body(Some("not-hex".into())))
        .await;
    assert_eq!(r.status().as_u16(), 400);

    // Well-formed but wrong (tampered) hash, and empty ("expect no row") on an
    // existing row: refused with the stale flash.
    let mut tampered = real.clone().into_bytes();
    tampered[0] = if tampered[0] == b'0' { b'1' } else { b'0' };
    for h in [String::from_utf8(tampered).unwrap(), String::new()] {
        let r = test_app
            .post_save_newsletter_draft_raw(&body(Some(h)))
            .await;
        assert_is_redirect_to(&r, &format!("/admin/newsletters/{issue_id}/edit"));
        let html = test_app.get_newsletter_edit_html(issue_id).await;
        assert!(html.contains(STALE_FLASH), "stale flash expected: {html}");
    }
    assert_eq!(stored_content(&test_app, issue_id).await.0, "orig");

    // Non-empty hash for an id with no row is refused and inserts nothing.
    let ghost = uuid::Uuid::new_v4();
    let r = test_app
        .post_save_newsletter_draft_raw(&serde_json::json!({
            "title": "ghost", "text_content": "t", "html_content": "h",
            "newsletter_issue_id": ghost.to_string(), "loaded_content_hash": real,
            "idempotency_key": uuid::Uuid::now_v7().to_string(),
        }))
        .await;
    assert_eq!(r.status().as_u16(), 303);
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1",
    )
    .bind(ghost)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
async fn malformed_newsletter_issue_id_in_compose_publish_is_400_and_writes_nothing() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let response = test_app
        .post_publish_newsletter(&serde_json::json!({
            "title": "x", "text_content": "y", "html_content": "z",
            "newsletter_issue_id": "not-a-uuid",
            "idempotency_key": uuid::Uuid::now_v7().to_string()
        }))
        .await;
    assert_eq!(response.status().as_u16(), 400);
    let (issues,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM newsletter.newsletter_issues")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(issues, 0, "no issue row");
    let (keys,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM idempotency")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(keys, 0, "no idempotency row");
}

#[tokio::test]
async fn malformed_publish_by_id_path_is_404_and_enqueues_nothing() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;
    let response = test_app
        .api_client
        .post(format!(
            "{}/admin/newsletters/not-a-uuid/publish",
            test_app.address
        ))
        .form(&serde_json::json!({ "idempotency_key": uuid::Uuid::now_v7().to_string() }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status().as_u16(), 404);
    let (queued,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM newsletter.issue_delivery_queue")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(queued, 0);
}

fn v7_at(unix_secs: u64) -> uuid::Uuid {
    uuid::Uuid::new_v7(uuid::Timestamp::from_unix(uuid::NoContext, unix_secs, 0))
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::test]
async fn idempotency_key_v7_is_accepted_on_publish() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let body = serde_json::json!({
        "title": "v7 ok",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": uuid::Uuid::now_v7().to_string()
    });
    let response = test_app.post_publish_newsletter(&body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM idempotency WHERE action = 'publish'")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(n, 1);
}

#[tokio::test]
async fn idempotency_key_v4_is_400_and_writes_nothing() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let body = serde_json::json!({
        "title": "v4 no",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = test_app.post_publish_newsletter(&body).await;
    assert_eq!(response.status().as_u16(), 400);
    let (issues,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM newsletter.newsletter_issues")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    let (keys,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM idempotency")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(issues, 0);
    assert_eq!(keys, 0);
}

#[tokio::test]
async fn idempotency_key_nil_is_400_and_writes_nothing() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let body = serde_json::json!({
        "title": "nil no",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": uuid::Uuid::nil().to_string()
    });
    let response = test_app.post_publish_newsletter(&body).await;
    assert_eq!(response.status().as_u16(), 400);
    let (keys,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM idempotency")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(keys, 0);
}

#[tokio::test]
async fn idempotency_key_malformed_is_400_and_writes_nothing() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let body = serde_json::json!({
        "title": "bad",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": "not-a-uuid"
    });
    let response = test_app.post_publish_newsletter(&body).await;
    assert_eq!(response.status().as_u16(), 400);
    let (keys,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM idempotency")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(keys, 0);
}

#[tokio::test]
async fn idempotency_key_stale_timestamp_is_400() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let stale = v7_at(unix_now() - 25 * 60 * 60);
    let body = serde_json::json!({
        "title": "stale",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": stale.to_string()
    });
    let response = test_app.post_publish_newsletter(&body).await;
    assert_eq!(response.status().as_u16(), 400);
    let (keys,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM idempotency")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(keys, 0);
}

#[tokio::test]
async fn idempotency_key_future_timestamp_is_400() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;
    let future = v7_at(unix_now() + 10 * 60);
    let body = serde_json::json!({
        "title": "future",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": future.to_string()
    });
    let response = test_app.post_publish_newsletter(&body).await;
    assert_eq!(response.status().as_u16(), 400);
    let (keys,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM idempotency")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(keys, 0);
}

#[tokio::test]
async fn duplicate_save_draft_is_deduped_and_not_refused_as_stale() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    let key = uuid::Uuid::now_v7().to_string();
    let body = serde_json::json!({
        "title": "First save",
        "text_content": "v1",
        "html_content": "<p>v1</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "loaded_content_hash": "",
        "idempotency_key": key
    });
    let response = test_app.post_save_newsletter_draft(&body).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));
    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(html.contains("Draft saved."), "first save flash: {html}");

    // Same key + same body (loaded hash still empty / original). Without replay-first
    // ordering this would look stale because the row now exists with different content.
    let response = test_app.post_save_newsletter_draft_raw(&body).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));
    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(
        html.contains("Draft saved."),
        "replay must flash first success, not stale: {html}"
    );
    assert!(
        !html.contains("changed since you opened it"),
        "duplicate save must not be refused as stale: {html}"
    );

    let title: String = sqlx::query_scalar!(
        r#"SELECT title FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1"#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(title, "First save");

    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM idempotency WHERE idempotency_key = $1 AND action = 'save_draft'",
    )
    .bind(&key)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(n, 1);
}

#[tokio::test]
async fn same_key_save_draft_with_different_content_is_refused_without_writing() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    let key = uuid::Uuid::now_v7().to_string();
    let body_a = serde_json::json!({
        "title": "Content A",
        "text_content": "a",
        "html_content": "<p>a</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "loaded_content_hash": "",
        "idempotency_key": key
    });
    let response = test_app.post_save_newsletter_draft(&body_a).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));
    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(html.contains("Draft saved."), "first save flash: {html}");

    // Same key and issue id, different content: the replay guard runs before the
    // R6a hash check, so the empty loaded hash is irrelevant.
    let body_b = serde_json::json!({
        "title": "Content B",
        "text_content": "b",
        "html_content": "<p>b</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "loaded_content_hash": "",
        "idempotency_key": key
    });
    let response = test_app.post_save_newsletter_draft_raw(&body_b).await;
    let html =
        assert_refused_to_list_with_error_flash(&test_app, response, &["Draft saved."]).await;
    assert!(
        html.contains("different content"),
        "same-key save with new content must be refused: {html}"
    );

    let (title, text_content, html_content): (String, String, String) = sqlx::query_as(
        "SELECT title, text_content, html_content FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(title, "Content A");
    assert_eq!(text_content, "a");
    assert_eq!(html_content, "<p>a</p>");

    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM idempotency WHERE idempotency_key = $1 AND action = 'save_draft'",
    )
    .bind(&key)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(n, 1);
}

/// Replays `key` (already used for draft `other_id`) against draft `issue_id` with
/// the given content and checks the R9 contract: the key is bound to the request
/// it was first used for, so the replay is refused in both content cases. The
/// response redirects to the list page with an error flash that does not claim
/// "Draft saved.", and no row is created for `issue_id`.
async fn assert_cross_id_key_reuse_contract(
    test_app: &TestApp,
    other_id: uuid::Uuid,
    issue_id: uuid::Uuid,
    key: &str,
    (title, text_content, html_content): (&str, &str, &str),
) {
    let body = serde_json::json!({
        "title": title,
        "text_content": text_content,
        "html_content": html_content,
        "newsletter_issue_id": issue_id.to_string(),
        "loaded_content_hash": "",
        "idempotency_key": key
    });
    let response = test_app.post_save_newsletter_draft_raw(&body).await;
    assert_refused_to_list_with_error_flash(test_app, response, &["Draft saved."]).await;

    let row_for_issue: Option<(String, String, String)> = sqlx::query_as(
        "SELECT title, text_content, html_content FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1",
    )
    .bind(issue_id)
    .fetch_optional(&test_app.pg_pool)
    .await
    .unwrap();
    assert!(
        row_for_issue.is_none(),
        "key reused from {other_id} on {issue_id} must not create or change a draft \
         for {issue_id}: {row_for_issue:?}"
    );
}

#[tokio::test]
async fn cross_draft_key_reuse_does_not_report_saved_for_unwritten_draft() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let draft_a = uuid::Uuid::now_v7();
    let draft_b = uuid::Uuid::now_v7();
    assert_ne!(draft_a, draft_b);
    let key = uuid::Uuid::now_v7().to_string();

    let body_a = serde_json::json!({
        "title": "Draft A",
        "text_content": "text a",
        "html_content": "<p>html a</p>",
        "newsletter_issue_id": draft_a.to_string(),
        "loaded_content_hash": "",
        "idempotency_key": key
    });
    let response = test_app.post_save_newsletter_draft(&body_a).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{draft_a}/edit"));
    let html = test_app.get_newsletter_edit_html(draft_a).await;
    assert!(html.contains("Draft saved."), "first save flash: {html}");

    let original_a = (
        "Draft A".to_owned(),
        "text a".to_owned(),
        "<p>html a</p>".to_owned(),
    );

    // Phase 1: same key, other id, different content: refused.
    assert_cross_id_key_reuse_contract(
        &test_app,
        draft_a,
        draft_b,
        &key,
        ("Draft B", "text b", "<p>html b</p>"),
    )
    .await;
    assert_eq!(stored_content(&test_app, draft_a).await, original_a);

    // Phase 2: same key, other id, content identical to A's: still refused. The
    // key is bound to A's request, so B must not be reported as saved.
    assert_cross_id_key_reuse_contract(
        &test_app,
        draft_a,
        draft_b,
        &key,
        ("Draft A", "text a", "<p>html a</p>"),
    )
    .await;
    assert_eq!(stored_content(&test_app, draft_a).await, original_a);

    // Neither refusal wrote or rewrote an idempotency row.
    assert_eq!(idempotency_rows_for(&test_app, &key, "save_draft").await, 1);
}

#[tokio::test]
async fn publish_by_id_key_reused_on_another_draft_is_refused_without_enqueue() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let draft_a = uuid::Uuid::new_v4();
    let draft_b = uuid::Uuid::new_v4();
    for (id, title) in [(draft_a, "Draft A"), (draft_b, "Draft B")] {
        let response = test_app
            .post_save_newsletter_draft(&serde_json::json!({
                "title": title,
                "text_content": "plain",
                "html_content": "<p>html</p>",
                "newsletter_issue_id": id.to_string(),
            }))
            .await;
        assert_is_redirect_to(&response, &format!("/admin/newsletters/{id}/edit"));
    }

    // Publish A under key K; consume the success flash so a stale cookie cannot
    // satisfy the later assertions.
    let key = uuid::Uuid::now_v7().to_string();
    let body = serde_json::json!({ "idempotency_key": key });
    let response = test_app.post_publish_saved_newsletter(draft_a, &body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains(SUCCESS_FLASH), "{html}");
    let rows_a = delivery_rows_for(&test_app, draft_a).await;
    assert_eq!(rows_a, 1);

    // Same key K, publish-by-id for the other draft B: a different request.
    let response = test_app.post_publish_saved_newsletter(draft_b, &body).await;
    assert_refused_to_list_with_error_flash(&test_app, response, &[SUCCESS_FLASH]).await;

    let (status_b, content_b, queued_b) = issue_status_and_queue(&test_app, draft_b).await;
    assert_eq!(status_b, "draft");
    assert_eq!(
        content_b,
        ("Draft B".into(), "plain".into(), "<p>html</p>".into())
    );
    assert_eq!(queued_b, 0);
    assert_eq!(delivery_rows_for(&test_app, draft_a).await, rows_a);
    assert_eq!(idempotency_rows_for(&test_app, &key, "publish").await, 1);

    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn compose_publish_key_reused_on_publish_by_id_is_refused_without_enqueue() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let issue_a = uuid::Uuid::new_v4();
    let key = uuid::Uuid::now_v7().to_string();
    let response = test_app
        .post_publish_newsletter(&serde_json::json!({
            "title": "Compose A",
            "text_content": "plain",
            "html_content": "<p>html</p>",
            "newsletter_issue_id": issue_a.to_string(),
            "idempotency_key": key
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains(SUCCESS_FLASH), "{html}");
    let rows_before = delivery_rows_for(&test_app, issue_a).await;
    assert_eq!(rows_before, 1);

    // Publish-by-id shares the `publish` action scope, so K finds the compose
    // Publish row; the endpoint differs, so it is a different request.
    let response = test_app
        .post_publish_saved_newsletter(issue_a, &serde_json::json!({ "idempotency_key": key }))
        .await;
    assert_refused_to_list_with_error_flash(&test_app, response, &[SUCCESS_FLASH]).await;

    assert_eq!(delivery_rows_for(&test_app, issue_a).await, rows_before);
    assert_eq!(idempotency_rows_for(&test_app, &key, "publish").await, 1);

    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn compose_publish_replay_with_different_issue_id_is_refused_without_enqueue() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // First request carries no issue id: the server mints one.
    let key = uuid::Uuid::now_v7().to_string();
    let first = serde_json::json!({
        "title": "Minted id",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": key
    });
    let response = test_app.post_publish_newsletter(&first).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains(SUCCESS_FLASH), "{html}");

    // Same key and content, but now an explicit id S the first request never had.
    let issue_s = uuid::Uuid::new_v4();
    let mut second = first.clone();
    second["newsletter_issue_id"] = issue_s.to_string().into();
    let response = test_app.post_publish_newsletter(&second).await;
    assert_refused_to_list_with_error_flash(&test_app, response, &[SUCCESS_FLASH]).await;

    assert_eq!(delivery_rows_for(&test_app, issue_s).await, 0);
    let (issue_s_rows,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1",
    )
    .bind(issue_s)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(issue_s_rows, 0, "refused replay must not create issue S");
    assert_eq!(count_newsletter_issues(&test_app).await, 1);
    assert_eq!(idempotency_rows_for(&test_app, &key, "publish").await, 1);

    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn identical_compose_publish_duplicate_without_issue_id_replays_success() {
    // Pins the dedupe that binding must not break: with no explicit id the first
    // request mints one server-side, so the fingerprint cannot depend on it.
    // Unlike duplicate_compose_publish_is_deduped_without_second_queue_rows, the
    // first flash is consumed before the duplicate, so the replay has to set its
    // own success flash.
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let key = uuid::Uuid::now_v7().to_string();
    let body = serde_json::json!({
        "title": "Same again",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": key
    });
    let response = test_app.post_publish_newsletter(&body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains(SUCCESS_FLASH), "first publish flash: {html}");

    let response = test_app.post_publish_newsletter(&body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(
        html.contains(SUCCESS_FLASH),
        "duplicate must replay: {html}"
    );

    assert_eq!(count_newsletter_issues(&test_app).await, 1);
    let (queue,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM newsletter.issue_delivery_queue")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert!(queue <= 1, "queue rows={queue}");
    assert_eq!(idempotency_rows_for(&test_app, &key, "publish").await, 1);

    test_app.dispatch_all_pending_emails().await;
}

#[tokio::test]
async fn legacy_saved_draft_row_without_fingerprint_replays_as_before() {
    // A row written before request binding carries no fingerprint header. Build
    // one by hand: a finished `save_draft` response (303 to the edit page, outcome
    // "saved") and nothing else. Replaying its key must keep returning the saved
    // response, not refuse it.
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let (user_id,): (uuid::Uuid,) = sqlx::query_as("SELECT user_id FROM users WHERE username = $1")
        .bind(&test_app.test_user.username)
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    // The draft the old save wrote, so the edit page the replay points at renders.
    let issue_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO newsletter.newsletter_issues \
         (newsletter_issue_id, title, text_content, html_content, published_at, status) \
         VALUES ($1, 'Seeded', 'seeded', '<p>seeded</p>', now(), 'draft')",
    )
    .bind(issue_id)
    .execute(&test_app.pg_pool)
    .await
    .unwrap();
    let key = uuid::Uuid::now_v7().to_string();
    let location = format!("/admin/newsletters/{issue_id}/edit");
    sqlx::query(
        r#"
        INSERT INTO idempotency (
            user_id, idempotency_key, action,
            response_status_code, response_headers, response_body, created_at
        )
        VALUES (
            $1, $2, 'save_draft',
            303,
            ARRAY[
                ROW('location'::text, $3::bytea)::header_pair,
                ROW('x-newsletter-draft-outcome'::text, 'saved'::bytea)::header_pair
            ],
            ''::bytea,
            now()
        )
        "#,
    )
    .bind(user_id)
    .bind(&key)
    .bind(location.as_bytes())
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    let response = test_app
        .post_save_newsletter_draft_raw(&serde_json::json!({
            "title": "Whatever",
            "text_content": "t",
            "html_content": "<p>t</p>",
            "newsletter_issue_id": issue_id.to_string(),
            "loaded_content_hash": "",
            "idempotency_key": key
        }))
        .await;
    assert_is_redirect_to(&response, &location);
    let html = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(
        html.contains("Draft saved."),
        "legacy row must replay its saved outcome: {html}"
    );

    // It was a replay: the submitted content was neither processed nor written.
    assert_eq!(
        stored_content(&test_app, issue_id).await,
        ("Seeded".into(), "seeded".into(), "<p>seeded</p>".into())
    );
    assert_eq!(idempotency_rows_for(&test_app, &key, "save_draft").await, 1);
}

#[tokio::test]
async fn duplicate_compose_publish_is_deduped_without_second_queue_rows() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let key = uuid::Uuid::now_v7().to_string();
    let body = serde_json::json!({
        "title": "Once only",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "idempotency_key": key
    });
    let response = test_app.post_publish_newsletter(&body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let response = test_app.post_publish_newsletter(&body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains("emails will go out shortly"), "{html}");
    test_app.dispatch_all_pending_emails().await;

    let (issues,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM newsletter.newsletter_issues")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    assert_eq!(issues, 1);
    let (queue,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM newsletter.issue_delivery_queue")
        .fetch_one(&test_app.pg_pool)
        .await
        .unwrap();
    // Worker may or may not have drained; at most one enqueue happened.
    assert!(queue <= 1, "queue rows={queue}");
    let (keys,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM idempotency WHERE idempotency_key = $1 AND action = 'publish'",
    )
    .bind(&key)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(keys, 1);
}

#[tokio::test]
async fn duplicate_publish_by_id_is_deduped_without_second_queue_rows() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let issue_id = uuid::Uuid::new_v4();
    test_app
        .post_save_newsletter_draft(&serde_json::json!({
            "title": "Ready",
            "text_content": "plain",
            "html_content": "<p>html</p>",
            "newsletter_issue_id": issue_id.to_string(),
            "loaded_content_hash": "",
            "idempotency_key": uuid::Uuid::now_v7().to_string()
        }))
        .await;

    let key = uuid::Uuid::now_v7().to_string();
    let body = serde_json::json!({ "idempotency_key": key });
    let response = test_app
        .post_publish_saved_newsletter(issue_id, &body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    let response = test_app
        .post_publish_saved_newsletter(issue_id, &body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains("emails will go out shortly"), "{html}");
    test_app.dispatch_all_pending_emails().await;

    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_ne!(status, site::domain::NewsletterIssueStatus::Draft);

    let (keys,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM idempotency WHERE idempotency_key = $1 AND action = 'publish'",
    )
    .bind(&key)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(keys, 1);
}

#[tokio::test]
async fn same_form_key_can_save_then_publish_via_action_scope() {
    // Edit page mints one key for both Save and Publish-by-id; action scoping must
    // allow save_draft then publish without colliding.
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let issue_id = uuid::Uuid::new_v4();
    let key = uuid::Uuid::now_v7().to_string();
    let save_body = serde_json::json!({
        "title": "Shared key",
        "text_content": "plain",
        "html_content": "<p>html</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "loaded_content_hash": "",
        "idempotency_key": key
    });
    let response = test_app.post_save_newsletter_draft(&save_body).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));

    let response = test_app
        .post_publish_saved_newsletter(issue_id, &serde_json::json!({ "idempotency_key": key }))
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains("emails will go out shortly"), "{html}");
    test_app.dispatch_all_pending_emails().await;

    let (save_keys,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM idempotency WHERE idempotency_key = $1 AND action = 'save_draft'",
    )
    .bind(&key)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    let (pub_keys,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM idempotency WHERE idempotency_key = $1 AND action = 'publish'",
    )
    .bind(&key)
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(save_keys, 1);
    assert_eq!(pub_keys, 1);
}
