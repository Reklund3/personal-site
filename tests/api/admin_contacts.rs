use crate::helpers::{assert_is_redirect_to, spawn_app};
use chrono::{Duration, TimeZone, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// Mirrors the PAGE_SIZE constant in src/routes/admin/contacts.rs.
const PAGE_SIZE: i64 = 50;

async fn insert_contact(
    pool: &PgPool,
    name: &str,
    email: &str,
    message: &str,
    contact_time: chrono::DateTime<Utc>,
) {
    sqlx::query!(
        r#"
        INSERT INTO contacts (id, email, name, message, contact_time)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        Uuid::new_v4(),
        email,
        name,
        message,
        contact_time,
    )
    .execute(pool)
    .await
    .expect("Failed to insert test contact.");
}

#[tokio::test]
async fn you_must_be_logged_in_to_see_contacts() {
    let test_app = spawn_app().await;

    let response = test_app.get_admin_contacts("").await;

    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn contacts_are_listed_after_login() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let base_time = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
    insert_contact(
        &test_app.pg_pool,
        "Ada Lovelace",
        "ada@example.com",
        "Hello from Ada!",
        base_time,
    )
    .await;
    insert_contact(
        &test_app.pg_pool,
        "Grace Hopper",
        "grace@example.com",
        "Hello from Grace!",
        base_time + Duration::seconds(1),
    )
    .await;

    let html_page = test_app.get_admin_contacts_html("").await;

    assert!(html_page.contains("Ada Lovelace"));
    assert!(html_page.contains("ada@example.com"));
    assert!(html_page.contains("Hello from Ada!"));
    assert!(html_page.contains("Grace Hopper"));
    assert!(html_page.contains("grace@example.com"));
    assert!(html_page.contains("Hello from Grace!"));
}

#[tokio::test]
async fn pagination_splits_across_pages() {
    let test_app = spawn_app().await;
    test_app.test_user.login(&test_app).await;

    let base_time = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let total_contacts = PAGE_SIZE + 1;
    for i in 0..total_contacts {
        insert_contact(
            &test_app.pg_pool,
            &format!("Contact {i}"),
            &format!("contact{i}@example.com"),
            &format!("Message {i}"),
            base_time + Duration::seconds(i),
        )
        .await;
    }

    // Newest contact_time is the last one inserted (index total_contacts - 1).
    let newest_index = total_contacts - 1;
    let oldest_index = 0;

    let page_one_html = test_app.get_admin_contacts_html("?page=1").await;
    assert!(page_one_html.contains(&format!("Contact {newest_index}")));
    assert!(!page_one_html.contains(&format!("Contact {oldest_index}")));

    let page_two_html = test_app.get_admin_contacts_html("?page=2").await;
    assert!(page_two_html.contains(&format!("Contact {oldest_index}")));
    assert!(!page_two_html.contains(&format!("Contact {newest_index}")));
}
