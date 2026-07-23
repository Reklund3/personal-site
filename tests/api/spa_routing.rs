use crate::helpers::spawn_app;

// Mirrors the SPA shell routes registered in src/startup.rs.
const SPA_PATHS: [&str; 5] = ["/", "/skills", "/experience", "/education", "/portfolio"];

#[tokio::test]
async fn known_spa_routes_serve_the_shell() {
    let test_app = spawn_app().await;

    for path in SPA_PATHS {
        let response = test_app
            .api_client
            .get(format!("{}{}", test_app.address, path))
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            response.status().as_u16(),
            200,
            "GET {path} should serve the SPA shell"
        );
    }
}

#[tokio::test]
async fn unknown_paths_return_404() {
    let test_app = spawn_app().await;

    for path in ["/summary", "/nonsense"] {
        let response = test_app
            .api_client
            .get(format!("{}{}", test_app.address, path))
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            response.status().as_u16(),
            404,
            "GET {path} should return 404"
        );
    }
}
