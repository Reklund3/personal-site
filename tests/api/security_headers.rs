use crate::helpers::spawn_app;

/// The same shell `home/mod.rs` embeds, so these tests scan exactly what is served.
static INDEX_HTML: &str = include_str!("../../ui/dist/index.html");

/// Pull one directive's source list out of a `Content-Security-Policy` header value.
fn csp_directive<'a>(policy: &'a str, name: &str) -> Option<&'a str> {
    policy.split(';').map(str::trim).find_map(|directive| {
        directive
            .strip_prefix(name)
            .filter(|rest| rest.starts_with(' '))
            .map(str::trim)
    })
}

async fn fetch_csp(app: &crate::helpers::TestApp) -> String {
    let response = reqwest::get(&app.address).await.expect("Failed to GET /");
    response
        .headers()
        .get("Content-Security-Policy")
        .expect("The home page must carry a Content-Security-Policy header")
        .to_str()
        .expect("Content-Security-Policy must be valid ASCII")
        .to_string()
}

/// script-src decides whether an injected `<script>` runs, and nothing here needs inline:
/// one external bundle, no inline handlers, and JSON-LD is an exempt data block.
#[tokio::test]
async fn csp_script_src_allows_no_inline_or_eval() {
    let app = spawn_app().await;

    let policy = fetch_csp(&app).await;
    let script_src = csp_directive(&policy, "script-src")
        .unwrap_or_else(|| panic!("CSP declares no script-src directive: {policy}"));

    assert!(
        !script_src.contains("'unsafe-inline'"),
        "script-src must not allow inline scripts, got: {script_src}"
    );
    assert!(
        !script_src.contains("'unsafe-eval'"),
        "script-src must not allow eval, got: {script_src}"
    );
}

/// The defect this file was written for: `'unsafe-inline'` covers inline `<style>`, never an
/// external stylesheet, so the webfont was blocked. Origins are derived from the shell so that
/// a new external stylesheet fails here rather than in a browser.
#[tokio::test]
async fn csp_style_src_permits_every_external_stylesheet_the_shell_loads() {
    let app = spawn_app().await;

    let policy = fetch_csp(&app).await;
    let style_src = csp_directive(&policy, "style-src")
        .unwrap_or_else(|| panic!("CSP declares no style-src directive: {policy}"));

    let origins = external_stylesheet_origins(INDEX_HTML);
    assert!(
        !origins.is_empty(),
        "Expected the shell to load at least one external stylesheet; if that is no longer \
         true, delete this test rather than weakening it"
    );

    for origin in origins {
        assert!(
            style_src.contains(&origin),
            "style-src must permit {origin}, which ui/dist/index.html loads a stylesheet from. \
             Got: {style_src}"
        );
    }
}

/// `fonts.gstatic.com` serves the woff2 files but appears nowhere in the markup — only in the
/// CSS `fonts.googleapis.com` returns. That indirection is why it was missed, and why this
/// origin is asserted explicitly instead of derived.
#[tokio::test]
async fn csp_font_src_permits_the_google_fonts_file_origin() {
    let app = spawn_app().await;

    let policy = fetch_csp(&app).await;
    let font_src = csp_directive(&policy, "font-src")
        .unwrap_or_else(|| panic!("CSP declares no font-src directive: {policy}"));

    assert!(
        INDEX_HTML.contains("fonts.googleapis.com"),
        "This test only applies while the shell loads Google Fonts"
    );
    assert!(
        font_src.contains("https://fonts.gstatic.com"),
        "font-src must permit the origin Google Fonts serves woff2 files from, got: {font_src}"
    );
}

/// Collect the origins of `<link rel="stylesheet">` hrefs that point off-origin.
fn external_stylesheet_origins(html: &str) -> Vec<String> {
    let mut origins = Vec::new();
    for tag in html.split('<').filter(|t| t.starts_with("link")) {
        let Some(tag) = tag.split('>').next() else {
            continue;
        };
        if !tag.contains("stylesheet") {
            continue;
        }
        let Some(href) = tag
            .split("href=\"")
            .nth(1)
            .and_then(|r| r.split('"').next())
        else {
            continue;
        };
        if let Some(rest) = href.strip_prefix("https://") {
            let origin = format!("https://{}", rest.split('/').next().unwrap_or_default());
            if !origins.contains(&origin) {
                origins.push(origin);
            }
        }
    }
    origins
}

/// A bare 200 is heuristically cacheable under RFC 9111, so without an explicit directive a
/// browser could re-serve these session-private pages from history after logout.
#[tokio::test]
async fn admin_pages_are_never_written_to_a_cache() {
    let app = spawn_app().await;
    app.test_user.login(&app).await;

    for (label, response) in [
        ("dashboard", app.get_admin_dashboard().await),
        ("contacts", app.get_admin_contacts("").await),
        ("newsletter form", app.get_publish_newsletter().await),
        ("password form", app.get_change_password().await),
    ] {
        assert!(
            response.status().is_success(),
            "The {label} page should have rendered for a signed-in user, got {}",
            response.status()
        );
        let cache_control = response
            .headers()
            .get("Cache-Control")
            .unwrap_or_else(|| panic!("The admin {label} page must set Cache-Control"))
            .to_str()
            .expect("Cache-Control must be valid ASCII");
        assert!(
            cache_control.contains("no-store"),
            "The admin {label} page must not be storable, got: {cache_control}"
        );
    }
}
