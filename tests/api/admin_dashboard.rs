use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    let test_app = spawn_app().await;

    let response = test_app.get_admin_dashboard().await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    let test_app = spawn_app().await;

    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
    });
    let response = test_app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    let html_page = test_app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", test_app.test_user.username)));

    let response = test_app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    let html_page = test_app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>You have successfully logged out.</i></p>"#));

    let response = test_app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn username_with_html_is_escaped_to_prevent_xss() {
    use crate::helpers::spawn_app;

    let test_app = spawn_app().await;

    // Create a malicious username with XSS attempt
    let malicious_username = "<script>alert('xss')</script>";

    // Insert a test user with malicious username directly into the database
    sqlx::query_unchecked!(
        r#"
        INSERT INTO users (user_id, username, password_hash)
        VALUES ($1, $2, $3)
        "#,
        uuid::Uuid::new_v4(),
        malicious_username,
        // Use a dummy password hash
        "$argon2id$v=19$m=15000,t=2,p=1$gZiV/M1gPc22ElAH/Jh1Hw$CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
    )
    .execute(&test_app.pg_pool)
    .await
    .expect("Failed to insert test user with malicious username");

    // Login with the regular test user
    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
    });
    let response = test_app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Get the admin dashboard HTML
    let html_page = test_app.get_admin_dashboard_html().await;

    // Verify that the username is present but HTML tags are escaped/removed
    // ammonia::clean() will strip the script tags entirely
    assert!(!html_page.contains("<script>"));
    assert!(!html_page.contains("alert('xss')"));

    // The page should still contain "Welcome" but without the malicious script
    assert!(html_page.contains("Welcome"));
}
