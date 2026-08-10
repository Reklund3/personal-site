use crate::helpers::spawn_app;

#[tokio::test]
async fn server_renders_route_specific_seo_metadata() {
    let test_app = spawn_app().await;

    let test_cases = [
        (
            "/",
            "<title>About Me | Robert Eklund</title>",
            r#"<meta name="description" content="Software engineer with expertise in Rust, functional programming, and distributed systems."#,
        ),
        (
            "/skills",
            "<title>Skills | Robert Eklund</title>",
            r#"<meta name="description" content="Technical skills including Rust, TypeScript, React, Scala, PostgreSQL, and DevOps tools."#,
        ),
        (
            "/experience",
            "<title>Experience | Robert Eklund</title>",
            r#"<meta name="description" content="Senior Micro-Service Engineer at Cloud Imperium Games."#,
        ),
        (
            "/education",
            "<title>Education | Robert Eklund</title>",
            r#"<meta name="description" content="Masters in Accounting Information Systems from Texas State University"#,
        ),
        (
            "/portfolio",
            "<title>Portfolio | Robert Eklund</title>",
            r#"<meta name="description" content="Personal projects and open source contributions."#,
        ),
    ];

    let mut titles = Vec::new();

    for (path, expected_title, expected_desc_substring) in test_cases {
        let response = test_app
            .api_client
            .get(format!("{}{}", test_app.address, path))
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            response.status().as_u16(),
            200,
            "GET {path} should return 200 OK"
        );

        let body = response
            .text()
            .await
            .expect("Failed to read response body.");

        assert!(
            body.contains(expected_title),
            "GET {path} should contain expected title '{expected_title}'"
        );

        assert!(
            body.contains(expected_desc_substring),
            "GET {path} should contain description substring '{expected_desc_substring}'"
        );

        let expected_canonical = format!(
            r#"<link rel="canonical" href="{}{path}" />"#,
            test_app.base_url
        );
        assert!(
            body.contains(&expected_canonical),
            "GET {path} should contain canonical link '{expected_canonical}'"
        );

        titles.push(expected_title);
    }

    // Assert that two different paths return different titles proving injection is happening per route
    assert_ne!(
        titles[0], titles[1],
        "Home page and Skills page must return different titles"
    );
}

/// The <h1> must be in the HTML the server sends, not only in the client bundle.
///
/// The SPA's page components each render an <h1>, but a crawler that does not execute JavaScript
/// never runs them: it sees the raw shell, whose body is a single empty <div id="root">. These
/// assertions are what keep the heading in the server-injected markup at the `<!--SSR-BODY-->`
/// marker, so the pages have a heading for the consumers that motivated adding one.
#[tokio::test]
async fn server_renders_an_h1_for_every_spa_route() {
    let test_app = spawn_app().await;

    let test_cases = [
        ("/", "<h1>About Me</h1>"),
        ("/skills", "<h1>Skills</h1>"),
        ("/experience", "<h1>Experience</h1>"),
        ("/education", "<h1>Education</h1>"),
        ("/portfolio", "<h1>Portfolio</h1>"),
    ];

    for (path, expected_h1) in test_cases {
        let response = test_app
            .api_client
            .get(format!("{}{}", test_app.address, path))
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            response.status().as_u16(),
            200,
            "GET {path} should return 200 OK"
        );

        let body = response
            .text()
            .await
            .expect("Failed to read response body.");

        assert!(
            body.contains(expected_h1),
            "GET {path} should contain the server-rendered heading '{expected_h1}'"
        );

        // Exactly one, not merely at least one: a second <h1> would give the page two competing
        // top-level headings once React mounts and renders its own.
        assert_eq!(
            body.matches("<h1").count(),
            1,
            "GET {path} should contain exactly one <h1>, found {}",
            body.matches("<h1").count()
        );
    }
}
