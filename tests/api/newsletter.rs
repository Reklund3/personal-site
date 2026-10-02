use crate::helpers::{ConfirmationLinks, TestApp, assert_is_redirect_to, spawn_app};
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
        "title": "ignored",
        "text_content": "ignored",
        "html_content": "<p>ignored</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
        "newsletter_issue_id": issue_id.to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
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
        "title": "ignored",
        "text_content": "ignored",
        "html_content": "<p>ignored</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
        "newsletter_issue_id": issue_id.to_string()
    });

    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains("already been handled"));

    // Replay with the same idempotency key must not flash the success message.
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
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

    let key_a = uuid::Uuid::new_v4().to_string();
    let first_a = serde_json::json!({
        "title": "ignored",
        "text_content": "ignored",
        "html_content": "<p>ignored</p>",
        "idempotency_key": key_a,
        "newsletter_issue_id": sent_id.to_string()
    });
    let response = test_app.post_publish_newsletter(&first_a).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains("already been handled"));

    // Replay with a mismatched body: no newsletter_issue_id (would have been
    // treated as a "new publish ⇒ success" under the old body-keyed logic).
    let replay_a = serde_json::json!({
        "title": "different",
        "text_content": "different",
        "html_content": "<p>different</p>",
        "idempotency_key": key_a
    });
    let response = test_app.post_publish_newsletter(&replay_a).await;
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("already been handled"),
        "replay flash must match first AlreadyHandled outcome, got: {html_page}"
    );
    assert!(!html_page.contains("emails will go out shortly"));

    // --- Case B: first new publish Queued; replay with an unrelated sent id ---
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let key_b = uuid::Uuid::new_v4().to_string();
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
    assert_is_redirect_to(&response, "/admin/newsletters");
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("emails will go out shortly"),
        "replay flash must match first Queued/success outcome, got: {html_page}"
    );
    assert!(
        !html_page.contains("already been handled"),
        "mismatched replay body must not switch flash to already-handled"
    );

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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
async fn http_accept_of_existing_draft_via_newsletter_issue_id() {
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
        VALUES ($1, 'Draft to accept', 'draft text', '<p>draft</p>', now(), 'draft')
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

    let newsletter_request_body = serde_json::json!({
        "title": "ignored on accept",
        "text_content": "ignored",
        "html_content": "<p>ignored</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
        "newsletter_issue_id": issue_id.to_string()
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

    // Content fields on the form must be ignored — original draft content kept.
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
    assert_eq!(title, "Draft to accept");

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
    assert_eq!(status, site::domain::NewsletterIssueStatus::Sent);
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
async fn unknown_newsletter_issue_id_returns_400_and_rolls_back_idempotency() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    let unknown_id = uuid::Uuid::new_v4();
    let idempotency_key = uuid::Uuid::new_v4().to_string();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&test_app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "ignored",
        "text_content": "ignored",
        "html_content": "<p>ignored</p>",
        "idempotency_key": idempotency_key,
        "newsletter_issue_id": unknown_id.to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_eq!(
        response.status().as_u16(),
        400,
        "missing issue must be a client error, not a saved redirect"
    );

    let queued = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        unknown_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued, Some(0), "no delivery rows for a missing issue");

    // App guarantee today: returning Err before save_response drops the
    // try_processing transaction, so the idempotency INSERT rolls back and the
    // same key can be used again (e.g. after correcting the issue id).
    let draft_id = uuid::Uuid::new_v4();
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
        draft_id
    )
    .execute(&test_app.pg_pool)
    .await
    .unwrap();

    // Same key after the 400 must start fresh processing (idempotency INSERT
    // rolled back with the failed request), not hit "expected a saved response".
    let retry_body = serde_json::json!({
        "title": "ignored",
        "text_content": "ignored",
        "html_content": "<p>ignored</p>",
        "idempotency_key": idempotency_key,
        "newsletter_issue_id": draft_id.to_string()
    });
    let retry = test_app.post_publish_newsletter(&retry_body).await;
    assert_is_redirect_to(&retry, "/admin/newsletters");

    let status = sqlx::query_scalar!(
        r#"
        SELECT status as "status: site::domain::NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        draft_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(status, site::domain::NewsletterIssueStatus::Queued);

    let queued_after = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM newsletter.issue_delivery_queue
        WHERE newsletter_issue_id = $1
        "#,
        draft_id
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(queued_after, Some(1));
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
