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
