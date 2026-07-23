use crate::email_client::ApplicationBaseUrl;
use actix_web::{HttpRequest, HttpResponse, Responder, http::header::ContentType, web::Data};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RouteMetadata {
    title: String,
    description: String,
    keywords: Option<String>,
    og_type: Option<String>,
    include_profile_tags: Option<bool>,
}

static ROUTES: LazyLock<HashMap<String, RouteMetadata>> = LazyLock::new(|| {
    let json_str = include_str!("../../../ui/src/seo/routes.json");
    serde_json::from_str(json_str).expect("Failed to parse routes.json")
});

static INDEX_HTML: &str = include_str!("../../../ui/dist/index.html");

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn build_seo_tags(path: &str, base_url: &str) -> String {
    let meta = ROUTES
        .get(path)
        .or_else(|| ROUTES.get("/"))
        .expect("Default route metadata missing");

    let full_title = format!("{} | Robert Eklund", meta.title);
    let escaped_title = html_escape(&full_title);
    let escaped_description = html_escape(&meta.description);
    let site_url = base_url.trim_end_matches('/');
    let canonical_url = format!("{}{}", site_url, path);
    let escaped_canonical = html_escape(&canonical_url);
    let headshot_url = format!("{}/headshot", site_url);
    let escaped_headshot = html_escape(&headshot_url);
    let og_type = meta.og_type.as_deref().unwrap_or("website");

    let mut tags = Vec::new();

    // Basic SEO
    tags.push(format!("<title>{}</title>", escaped_title));
    tags.push(format!(
        r#"<meta name="description" content="{}" />"#,
        escaped_description
    ));
    if let Some(keywords) = &meta.keywords {
        tags.push(format!(
            r#"<meta name="keywords" content="{}" />"#,
            html_escape(keywords)
        ));
    }
    tags.push(r#"<meta name="author" content="Robert Eklund" />"#.to_string());
    tags.push(format!(
        r#"<link rel="canonical" href="{}" />"#,
        escaped_canonical
    ));

    // OpenGraph Tags
    tags.push(format!(
        r#"<meta property="og:title" content="{}" />"#,
        escaped_title
    ));
    tags.push(format!(
        r#"<meta property="og:description" content="{}" />"#,
        escaped_description
    ));
    tags.push(format!(
        r#"<meta property="og:type" content="{}" />"#,
        html_escape(og_type)
    ));
    tags.push(format!(
        r#"<meta property="og:url" content="{}" />"#,
        escaped_canonical
    ));
    tags.push(format!(
        r#"<meta property="og:image" content="{}" />"#,
        escaped_headshot
    ));
    tags.push(r#"<meta property="og:site_name" content="Robert Eklund Portfolio" />"#.to_string());

    if meta.include_profile_tags.unwrap_or(false) {
        tags.push(r#"<meta property="profile:first_name" content="Robert" />"#.to_string());
        tags.push(r#"<meta property="profile:last_name" content="Eklund" />"#.to_string());
    }

    // Twitter Card Tags
    tags.push(r#"<meta name="twitter:card" content="summary" />"#.to_string());
    tags.push(format!(
        r#"<meta name="twitter:title" content="{}" />"#,
        escaped_title
    ));
    tags.push(format!(
        r#"<meta name="twitter:description" content="{}" />"#,
        escaped_description
    ));
    tags.push(format!(
        r#"<meta name="twitter:image" content="{}" />"#,
        escaped_headshot
    ));

    // Person JSON-LD Schema (on profile/about page)
    if meta.include_profile_tags.unwrap_or(false) {
        let json_ld = serde_json::json!({
            "@context": "https://schema.org",
            "@type": "Person",
            "name": "Robert Eklund",
            "jobTitle": "Software Engineer",
            "url": site_url,
            "image": headshot_url,
            "sameAs": [
                "https://github.com/Reklund3",
                "https://www.linkedin.com/in/robert-eklund-64302976/"
            ],
            "knowsAbout": [
                "Rust",
                "Functional Programming",
                "TypeScript",
                "React",
                "Distributed Systems",
                "Actix Web",
                "PostgreSQL"
            ],
            "alumniOf": {
                "@type": "Organization",
                "name": "Texas State University"
            },
            "description": meta.description
        });
        tags.push(format!(
            r#"<script type="application/ld+json">{}</script>"#,
            json_ld
        ));
    }

    tags.join("\n    ")
}

pub async fn home(req: HttpRequest, base_url: Data<ApplicationBaseUrl>) -> impl Responder {
    let body = if INDEX_HTML.contains("<!--SEO-->") {
        let seo_fragment = build_seo_tags(req.path(), base_url.as_ref().as_ref());
        INDEX_HTML.replace("<!--SEO-->", &seo_fragment)
    } else {
        INDEX_HTML.to_string()
    };

    HttpResponse::Ok()
        .content_type(ContentType::html())
        .insert_header(("Cache-Control", "max-age=3600, must-revalidate"))
        .body(body)
}
