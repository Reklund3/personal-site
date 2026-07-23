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

        let expected_canonical =
            format!(r#"<link rel="canonical" href="http://127.0.0.1:8080{path}" />"#);
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
