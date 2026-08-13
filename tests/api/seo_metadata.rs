use crate::helpers::spawn_app;

#[tokio::test]
async fn server_renders_identical_seo_metadata_for_all_spa_paths() {
    let test_app = spawn_app().await;

    let test_cases = ["/", "/skills", "/experience", "/education", "/portfolio"];

    let expected_title = "<title>Software Engineer | Robert Eklund</title>";
    let expected_desc_substring = r#"<meta name="description" content="Software engineer with expertise in Rust, functional programming, TypeScript, and distributed systems."#;

    let mut titles = Vec::new();

    for path in test_cases {
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
            format!(r#"<link rel="canonical" href="{}/" />"#, test_app.base_url);
        assert!(
            body.contains(&expected_canonical),
            "GET {path} should contain canonical link '{expected_canonical}'"
        );

        let actual_title = body
            .lines()
            .find(|line| line.contains("<title>"))
            .expect("Response body should contain <title>")
            .trim()
            .to_string();
        titles.push(actual_title);
    }

    // Every path must return a byte-identical title. This is the inverted form of the
    // pre-one-pager `assert_ne!`, which proved metadata was injected *per route*; under a
    // single canonical the equivalent proof is that no path differs from any other.
    // Compare all of them, not just the first two — a regression on `/portfolio` alone
    // would otherwise pass.
    assert_eq!(
        titles.len(),
        test_cases.len(),
        "should have collected one title per path"
    );
    for (path, title) in test_cases.iter().zip(titles.iter()) {
        assert_eq!(
            title, &titles[0],
            "GET {path} returned a different title than {}; all SPA paths must be identical",
            test_cases[0]
        );
    }
}

/// The <h1> must be in the HTML the server sends, not only in the client bundle.
///
/// The SPA's one-pager renders an <h1>, but a crawler that does not execute JavaScript
/// never runs it: it sees the raw shell, whose body is a single empty <div id="root">. These
/// assertions are what keep the heading in the server-injected markup at the `<!--SSR-BODY-->`
/// marker, so the pages have a heading for the consumers that motivated adding one.
#[tokio::test]
async fn server_renders_an_h1_for_every_spa_route() {
    let test_app = spawn_app().await;

    let test_cases = ["/", "/skills", "/experience", "/education", "/portfolio"];

    let expected_h1 = "<h1>Robert Eklund</h1>";

    for path in test_cases {
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
