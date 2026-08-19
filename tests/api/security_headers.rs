use crate::helpers::spawn_app;

/// The HTML shell the server embeds at compile time (`src/routes/home/mod.rs` does the same
/// `include_str!`), so what these tests scan is byte-identical to what is served.
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

/// `'unsafe-inline'` in script-src is the single directive that decides whether an injected
/// `<script>` runs. Nothing in this app needs it: the Vite build emits one external module
/// bundle and no handler renders an inline handler or a `javascript:` URL. The JSON-LD block
/// in `home/mod.rs` is not an exception — a `type="application/ld+json"` script is a data
/// block, and HTML's "prepare the script element" steps return before the CSP check.
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

/// Guards the defect this test file was written for: the shell loads its webfont stylesheet
/// from another origin, and a style-src of `'self' 'unsafe-inline'` silently blocked it —
/// `'unsafe-inline'` covers inline `<style>`, never an external stylesheet. Deriving the
/// origins from the shell rather than hardcoding them means adding a new external stylesheet
/// without widening the CSP fails here instead of in a browser.
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

/// `fonts.googleapis.com` serves only CSS; the woff2 files it points at come from
/// `fonts.gstatic.com`, which appears nowhere in the shell's markup. That indirection is
/// exactly why the font origin was missed: nothing in the HTML names it, so font-src has to
/// be asserted explicitly.
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

/// Every page under /admin renders data belonging to one signed-in session. A bare 200 carries
/// no freshness information and RFC 9111 lets a cache invent one, so without an explicit
/// directive a browser may write these pages to its disk cache and re-serve them from session
/// history after logout.
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
