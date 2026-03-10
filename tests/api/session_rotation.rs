use crate::helpers::{assert_is_redirect_to, spawn_app};
use uuid::Uuid;

#[tokio::test]
async fn changing_password_rotates_session_id() {
    let test_app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();

    // Login
    let login_body = serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
    });
    let response = test_app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Get the session cookie after login
    let session_cookie_after_login = response
        .cookies()
        .find(|c| c.name() == "roberteklund")
        .expect("Session cookie not found after login");

    // Change password
    let response = test_app
        .post_change_password(&serde_json::json!({
            "current_password": &test_app.test_user.password,
            "new_password": &new_password,
            "new_password_check": &new_password,
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/password");

    // Get the session cookie after password change
    let session_cookie_after_password_change = response
        .cookies()
        .find(|c| c.name() == "roberteklund")
        .expect("Session cookie not found after password change");

    // Assert that the session ID has changed
    assert_ne!(
        session_cookie_after_login.value(),
        session_cookie_after_password_change.value(),
        "Session ID should have rotated after password change"
    );

    // Verify we can still access the dashboard with the new session
    let response = test_app.get_admin_dashboard().await;
    assert_eq!(response.status().as_u16(), 200);
}
