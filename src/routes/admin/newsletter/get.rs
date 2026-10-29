use crate::domain::NewsletterIssueStatus;
use crate::utils::e500;
use actix_web::HttpResponse;
use actix_web::http::header::ContentType;
use actix_web::web;
use actix_web_flash_messages::IncomingFlashMessages;
use sqlx::PgPool;
use std::fmt::Write;
use uuid::Uuid;

fn escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn flash_html(flash_messages: &IncomingFlashMessages) -> String {
    let mut msg_html = String::new();
    for m in flash_messages.iter() {
        // Flashes are server-authored today; escape anyway so a future
        // user-influenced message cannot become HTML.
        writeln!(msg_html, "<p><i>{}</i></p>", escape_attr(m.content())).unwrap();
    }
    msg_html
}

fn status_label(status: NewsletterIssueStatus) -> &'static str {
    match status {
        NewsletterIssueStatus::Draft => "draft",
        NewsletterIssueStatus::Queued => "queued",
        NewsletterIssueStatus::Sent => "sent",
        NewsletterIssueStatus::Failed => "failed",
    }
}

/// List drafts (with Edit links) and recent non-draft issues.
pub async fn list_newsletters(
    flash_messages: IncomingFlashMessages,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let msg_html = flash_html(&flash_messages);

    let drafts = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, title, published_at
        FROM newsletter.newsletter_issues
        WHERE status = 'draft'
        ORDER BY published_at DESC
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(e500)?;

    let recent = sqlx::query!(
        r#"
        SELECT
            newsletter_issue_id,
            title,
            status as "status: NewsletterIssueStatus",
            published_at
        FROM newsletter.newsletter_issues
        WHERE status <> 'draft'
        ORDER BY published_at DESC
        LIMIT 20
        "#
    )
    .fetch_all(pool.get_ref())
    .await
    .map_err(e500)?;

    let mut drafts_html = String::new();
    if drafts.is_empty() {
        drafts_html.push_str("<p><i>No drafts.</i></p>");
    } else {
        drafts_html.push_str("<ul>");
        for d in &drafts {
            let title = escape_attr(&d.title);
            let id = d.newsletter_issue_id;
            writeln!(
                drafts_html,
                r#"<li><a href="/admin/newsletters/{id}/edit">{title}</a></li>"#
            )
            .unwrap();
        }
        drafts_html.push_str("</ul>");
    }

    let mut recent_html = String::new();
    if recent.is_empty() {
        recent_html.push_str("<p><i>No published issues yet.</i></p>");
    } else {
        recent_html.push_str("<ul>");
        for r in &recent {
            let title = escape_attr(&r.title);
            let id = r.newsletter_issue_id;
            let status = status_label(r.status);
            writeln!(
                recent_html,
                r#"<li><a href="/admin/newsletters/{id}/edit">{title}</a> — {status}</li>"#
            )
            .unwrap();
        }
        recent_html.push_str("</ul>");
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>Newsletters</title>
</head>
<body>
    {msg_html}
    <h1>Newsletters</h1>
    <p><a href="/admin/newsletters/new">New issue</a></p>
    <h2>Drafts</h2>
    {drafts_html}
    <h2>Recent issues</h2>
    {recent_html}
    <p><a href="/admin/dashboard">&lt;- Back</a></p>
</body>
</html>"#
        )))
}

/// Compose a new issue: mints `newsletter_issue_id` + idempotency key for the form.
pub async fn publish_newsletter_form(
    flash_messages: IncomingFlashMessages,
) -> Result<HttpResponse, actix_web::Error> {
    let issue_id = Uuid::new_v4();
    let idempotency_key = Uuid::new_v4();
    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(render_issue_form(
            flash_html(&flash_messages),
            "New Newsletter Issue",
            issue_id,
            idempotency_key,
            "",
            "",
            "",
            FormMode::Editable,
            None,
        )))
}

#[derive(serde::Deserialize)]
pub struct EditPath {
    newsletter_issue_id: Uuid,
}

/// Prefill the form from a draft, or show a read-only view for queued/sent/failed.
pub async fn edit_newsletter_form(
    path: web::Path<EditPath>,
    flash_messages: IncomingFlashMessages,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let issue_id = path.newsletter_issue_id;
    let row = sqlx::query!(
        r#"
        SELECT
            title,
            text_content,
            html_content,
            status as "status: NewsletterIssueStatus"
        FROM newsletter.newsletter_issues
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(e500)?;

    let Some(row) = row else {
        return Ok(HttpResponse::NotFound()
            .content_type(ContentType::html())
            .body("Newsletter issue not found."));
    };

    let mode = match row.status {
        NewsletterIssueStatus::Draft => FormMode::Editable,
        other => FormMode::ReadOnly {
            status: status_label(other),
        },
    };
    let title_text = match mode {
        FormMode::Editable => "Edit Draft",
        FormMode::ReadOnly { .. } => "Newsletter Issue",
    };
    // Only editable forms need a publish idempotency key.
    let idempotency_key = match mode {
        FormMode::Editable => Uuid::new_v4(),
        FormMode::ReadOnly { .. } => Uuid::nil(),
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(render_issue_form(
            flash_html(&flash_messages),
            title_text,
            issue_id,
            idempotency_key,
            &row.title,
            &row.text_content,
            &row.html_content,
            mode,
            Some(status_label(row.status)),
        )))
}

enum FormMode {
    Editable,
    ReadOnly { status: &'static str },
}

#[allow(clippy::too_many_arguments)]
fn render_issue_form(
    msg_html: String,
    page_title: &str,
    issue_id: Uuid,
    idempotency_key: Uuid,
    title: &str,
    text_content: &str,
    html_content: &str,
    mode: FormMode,
    status_hint: Option<&str>,
) -> String {
    let title_v = escape_attr(title);
    let text_v = escape_attr(text_content);
    let html_v = escape_attr(html_content);

    let status_banner = match (&mode, status_hint) {
        (FormMode::ReadOnly { status }, _) => {
            format!("<p><b>Status:</b> {status} (read-only)</p>")
        }
        (FormMode::Editable, Some(s)) => format!("<p><b>Status:</b> {s}</p>"),
        (FormMode::Editable, None) => String::new(),
    };

    // Read-only issues must not render a POST form: `readonly` inputs can still
    // submit via Enter in some browsers, which would mint a forever idempotency
    // row for an AlreadyHandled publish. Show plain fields instead.
    let body_inner = match mode {
        FormMode::Editable => format!(
            r#"
    <form action="/admin/newsletters" method="post">
        <label>Title:<br>
            <input
                type="text"
                placeholder="Enter the issue title"
                name="title"
                value="{title_v}"
            >
        </label>
        <br>
        <label>Plain text content:<br>
            <textarea
                placeholder="Enter the content in plain text"
                name="text_content"
                rows="20"
                cols="50"
            >{text_v}</textarea>
        </label>
        <br>
        <label>HTML content:<br>
            <textarea
                placeholder="Enter the content in HTML format"
                name="html_content"
                rows="20"
                cols="50"
            >{html_v}</textarea>
        </label>
        <br>
        <input hidden type="text" name="idempotency_key" value="{idempotency_key}">
        <input hidden type="text" name="newsletter_issue_id" value="{issue_id}">
        <button type="submit" formaction="/admin/newsletters/drafts" formmethod="post">Save draft</button>
        <button type="submit">Publish</button>
    </form>"#
        ),
        FormMode::ReadOnly { .. } => format!(
            r#"
    <p><b>Title:</b> {title_v}</p>
    <p><b>Plain text content:</b></p>
    <pre>{text_v}</pre>
    <p><b>HTML content:</b></p>
    <pre>{html_v}</pre>"#
        ),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
    <title>{page_title}</title>
</head>
<body>
    {msg_html}
    <h1>{page_title}</h1>
    {status_banner}
    {body_inner}
    <p><a href="/admin/newsletters">&lt;- Newsletters</a></p>
</body>
</html>"#
    )
}
