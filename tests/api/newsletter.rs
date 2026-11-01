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
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
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
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
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
            &serde_json::json!({ "idempotency_key": uuid::Uuid::new_v4().to_string() }),
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
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
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
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
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
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
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
                "idempotency_key": uuid::Uuid::new_v4().to_string(),
                "newsletter_issue_id": uuid::Uuid::new_v4().to_string()
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "Title",
                "html_content": "<p>html</p>",
                "idempotency_key": uuid::Uuid::new_v4().to_string(),
                "newsletter_issue_id": uuid::Uuid::new_v4().to_string()
            }),
            "missing text content",
        ),
        (
            serde_json::json!({
                "title": "Title",
                "text_content": "text",
                "idempotency_key": uuid::Uuid::new_v4().to_string(),
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = test_app.post_save_newsletter_draft(&save_body).await;
    assert_is_redirect_to(&response, &format!("/admin/newsletters/{issue_id}/edit"));

    let edit_body = serde_json::json!({
        "title": "Edited title",
        "text_content": "edited plain",
        "html_content": "<p>edited</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
            &serde_json::json!({ "idempotency_key": uuid::Uuid::new_v4().to_string() }),
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
        SELECT title, text_content, status as "status: site::domain::NewsletterIssueStatus"
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = app.post_save_newsletter_draft(&body).await;
    assert_is_redirect_to(&response, "/login");

    let response = app
        .post_publish_saved_newsletter(
            issue_id,
            &serde_json::json!({ "idempotency_key": uuid::Uuid::new_v4().to_string() }),
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
            "idempotency_key": uuid::Uuid::new_v4().to_string()
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
    assert!(
        html.contains(r#"name="idempotency_key""#),
        "publish-by-id form needs an idempotency key"
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
    let nasty_title = r#""><script>alert(1)</script><"#;
    let nasty_text = "</textarea><script>alert(1)</script>";
    let nasty_html = r#"<img src=x onerror="alert(1)">"#;
    let body = serde_json::json!({
        "title": nasty_title,
        "text_content": nasty_text,
        "html_content": nasty_html,
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    test_app.post_save_newsletter_draft(&body).await;

    let edit = test_app.get_newsletter_edit_html(issue_id).await;
    assert!(
        edit.contains("&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;&lt;"),
        "title must be attribute-escaped: {edit}"
    );
    assert!(
        edit.contains("&lt;/textarea&gt;&lt;script&gt;alert(1)&lt;/script&gt;"),
        "textarea body must escape closers: {edit}"
    );
    assert!(
        !edit.contains("</textarea><script>"),
        "raw textarea breakout must not appear"
    );
    assert!(
        edit.contains("&lt;img src=x onerror=&quot;alert(1)&quot;&gt;"),
        "html_content must be escaped in the form: {edit}"
    );

    let list = test_app.get_publish_newsletter_html().await;
    assert!(
        list.contains("&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;&lt;"),
        "list titles must be escaped: {list}"
    );
    assert!(
        !list.contains("<script>alert(1)</script>"),
        "raw script must not appear in list HTML"
    );
}

#[tokio::test]
async fn draft_save_does_not_write_idempotency_row() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let issue_id = uuid::Uuid::new_v4();
    let key = uuid::Uuid::new_v4();
    let body = serde_json::json!({
        "title": "No idem row",
        "text_content": "x",
        "html_content": "<p>x</p>",
        "newsletter_issue_id": issue_id.to_string(),
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

    let idem_count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) FROM idempotency WHERE idempotency_key = $1"#,
        key.to_string()
    )
    .fetch_one(&test_app.pg_pool)
    .await
    .unwrap();
    assert_eq!(idem_count, Some(0));
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
    let status = response.status().as_u16();
    assert!(
        status == 400 || status == 404,
        "malformed path UUID should be 400 or 404, got {status}"
    );
}

#[tokio::test]
async fn concurrent_draft_save_and_publish_saved_never_false_draft_saved_on_non_draft() {
    let test_app = spawn_app().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    // Seed a draft first so publish-saved has a row to lock; then race a content
    // save against publish-by-id. Whichever wins, queued content must equal the
    // committed row (no mixed/stale compose body), and a late save is rejected.
    for _ in 0..8 {
        let issue_id = uuid::Uuid::new_v4();
        let seed = serde_json::json!({
            "title": "Seed title",
            "text_content": "seed plain",
            "html_content": "<p>seed</p>",
            "newsletter_issue_id": issue_id.to_string(),
            "idempotency_key": uuid::Uuid::new_v4().to_string()
        });
        test_app.post_save_newsletter_draft(&seed).await;

        let draft_body = serde_json::json!({
            "title": "Draft race",
            "text_content": "draft plain",
            "html_content": "<p>draft</p>",
            "newsletter_issue_id": issue_id.to_string(),
            "idempotency_key": uuid::Uuid::new_v4().to_string()
        });
        let publish_body = serde_json::json!({
            "idempotency_key": uuid::Uuid::new_v4().to_string()
        });

        let draft_fut = test_app.post_save_newsletter_draft(&draft_body);
        let publish_fut = test_app.post_publish_saved_newsletter(issue_id, &publish_body);
        let (draft_resp, publish_resp) = tokio::join!(draft_fut, publish_fut);

        assert_eq!(draft_resp.status().as_u16(), 303);
        assert_eq!(publish_resp.status().as_u16(), 303);

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

        // Content must be one of the committed writers — never a mix, and never
        // inventing compose-style publish body fields (there are none).
        let allowed = [
            ("Seed title", "seed plain", "<p>seed</p>"),
            ("Draft race", "draft plain", "<p>draft</p>"),
        ];
        assert!(
            allowed.contains(&(
                row.title.as_str(),
                row.text_content.as_str(),
                row.html_content.as_str()
            )),
            "unexpected mixed/unknown content: title={:?} text={:?} html={:?}",
            row.title,
            row.text_content,
            row.html_content
        );

        let html = test_app.get_newsletter_edit_html(issue_id).await;
        if row.status != site::domain::NewsletterIssueStatus::Draft {
            assert!(
                !html.contains("Draft saved."),
                "false Draft saved flash while status={:?}: {html}",
                row.status
            );
            // Late save after queue must be rejected.
            let late = test_app
                .post_save_newsletter_draft(&serde_json::json!({
                    "title": "Too late",
                    "text_content": "nope",
                    "html_content": "<p>nope</p>",
                    "newsletter_issue_id": issue_id.to_string(),
                    "idempotency_key": uuid::Uuid::new_v4().to_string()
                }))
                .await;
            assert_eq!(late.status().as_u16(), 303);
            let after = sqlx::query_scalar!(
                r#"SELECT title FROM newsletter.newsletter_issues WHERE newsletter_issue_id = $1"#,
                issue_id
            )
            .fetch_one(&test_app.pg_pool)
            .await
            .unwrap();
            assert_ne!(after, "Too late");
        }
    }
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
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    test_app.post_save_newsletter_draft(&v1).await;

    let v2 = serde_json::json!({
        "title": "Version Two",
        "text_content": "plain v2",
        "html_content": "<p>v2</p>",
        "newsletter_issue_id": issue_id.to_string(),
        "idempotency_key": uuid::Uuid::new_v4().to_string()
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
            &serde_json::json!({ "idempotency_key": uuid::Uuid::new_v4().to_string() }),
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
    let key = uuid::Uuid::new_v4().to_string();
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
            &serde_json::json!({ "idempotency_key": uuid::Uuid::new_v4().to_string() }),
        )
        .await;
    assert_eq!(response.status().as_u16(), 404);
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
            "idempotency_key": uuid::Uuid::new_v4().to_string()
        }))
        .await;
    let html = test_app.get_publish_newsletter_html().await;
    assert!(html.contains("(untitled)"));
    assert!(html.contains(&format!("/admin/newsletters/{issue_id}/edit")));
    assert!(!html.contains(&format!(
        r#"<a href="/admin/newsletters/{issue_id}/edit"></a>"#
    )));
}
