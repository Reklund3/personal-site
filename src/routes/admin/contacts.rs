use crate::utils::e500;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, web};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

const PAGE_SIZE: i64 = 50;

#[derive(serde::Deserialize)]
pub struct ContactsQueryParams {
    page: Option<i64>,
}

struct ContactRow {
    id: Uuid,
    email: String,
    name: String,
    message: Option<String>,
    contact_time: chrono::DateTime<chrono::Utc>,
}

pub async fn admin_contacts(
    query: web::Query<ContactsQueryParams>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let page = query.page.unwrap_or(1).max(1);

    let total_contacts = count_contacts(&pool).await.map_err(e500)?;
    let total_pages = (total_contacts + PAGE_SIZE - 1) / PAGE_SIZE;
    let total_pages = total_pages.max(1);
    let page = page.min(total_pages);
    let offset = (page - 1) * PAGE_SIZE;

    let contacts = fetch_contacts_page(&pool, PAGE_SIZE, offset)
        .await
        .map_err(e500)?;

    let contacts_html: String = contacts
        .iter()
        .map(|contact| {
            let safe_name = ammonia::clean(&contact.name);
            let safe_email = ammonia::clean(&contact.email);
            let safe_message = ammonia::clean(contact.message.as_deref().unwrap_or(""));
            let formatted_time = contact.contact_time.format("%Y-%m-%d %H:%M UTC");
            let contact_id = contact.id;
            format!(
                r#"<li id="contact-{contact_id}">
        <p><strong>{safe_name}</strong> &lt;{safe_email}&gt;</p>
        <p><em>{formatted_time}</em></p>
        <p>{safe_message}</p>
    </li>"#
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let prev_link = if page > 1 {
        format!(r#"<a href="/admin/contacts?page={}">Prev</a>"#, page - 1)
    } else {
        "Prev".to_string()
    };
    let next_link = if page < total_pages {
        format!(r#"<a href="/admin/contacts?page={}">Next</a>"#, page + 1)
    } else {
        "Next".to_string()
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Contacts</title>
</head>
<body>
    <h1>Contacts</h1>
    <ol>
{contacts_html}
    </ol>
    <p>Page {page} of {total_pages} &nbsp; {prev_link} &nbsp; {next_link}</p>
    <p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#,
        )))
}

#[tracing::instrument(name = "Count contacts", skip(pool))]
async fn count_contacts(pool: &PgPool) -> Result<i64, anyhow::Error> {
    let row = sqlx::query!(r#"SELECT COUNT(*) as "count!" FROM contacts"#)
        .fetch_one(pool)
        .await
        .context("Failed to perform a query to count contacts.")?;
    Ok(row.count)
}

#[tracing::instrument(name = "Fetch a page of contacts", skip(pool))]
async fn fetch_contacts_page(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<ContactRow>, anyhow::Error> {
    let rows = sqlx::query_as!(
        ContactRow,
        r#"
        SELECT id, email, name, message, contact_time
        FROM contacts
        ORDER BY contact_time DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await
    .context("Failed to perform a query to retrieve contacts.")?;
    Ok(rows)
}
