use crate::helpers::spawn_app;

const IMMUTABLE_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

/// Finds a real, content-hashed filename produced by the Vite build so the test doesn't
/// hardcode a hash that changes on every build.
fn find_hashed_asset_filename() -> String {
    let assets_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/ui/dist/assets");
    let entry = std::fs::read_dir(assets_dir)
        .unwrap_or_else(|e| {
            panic!("Failed to read {assets_dir}: {e}. Run `npm run build` in ui/ first.")
        })
        .filter_map(|entry| entry.ok())
        .find(|entry| entry.path().extension().is_some_and(|ext| ext == "js"))
        .unwrap_or_else(|| panic!("No .js asset found in {assets_dir}"));

    entry
        .file_name()
        .into_string()
        .expect("Asset filename is not valid UTF-8")
}

#[tokio::test]
async fn hashed_asset_is_served_with_immutable_cache_control() {
    let test_app = spawn_app().await;
    let asset_filename = find_hashed_asset_filename();

    let response = test_app
        .api_client
        .get(format!("{}/assets/{}", test_app.address, asset_filename))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        response.status().as_u16(),
        200,
        "GET /assets/{asset_filename} should return 200"
    );
    assert_eq!(
        response.headers().get("cache-control").unwrap(),
        IMMUTABLE_CACHE_CONTROL,
        "Hashed assets should be cached forever since a new build produces a new filename"
    );
}

#[tokio::test]
async fn conditional_revalidation_304_still_carries_immutable_cache_control() {
    let test_app = spawn_app().await;
    let asset_filename = find_hashed_asset_filename();
    let url = format!("{}/assets/{}", test_app.address, asset_filename);

    // First request obtains the validator (ETag) actix-files emits for the asset.
    let first = test_app
        .api_client
        .get(&url)
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(first.status().as_u16(), 200);
    let etag = first
        .headers()
        .get("etag")
        .expect("actix-files should set an ETag on hashed assets")
        .to_str()
        .unwrap()
        .to_owned();

    // A matching If-None-Match makes actix-files return 304 Not Modified. That 304 must still
    // carry the immutable Cache-Control, otherwise a revalidating client/CDN would lose the
    // directive. This locks in the explicit NOT_MODIFIED branch in the /assets middleware.
    let revalidated = test_app
        .api_client
        .get(&url)
        .header("If-None-Match", etag)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        revalidated.status().as_u16(),
        304,
        "A matching If-None-Match should return 304 Not Modified"
    );
    assert_eq!(
        revalidated
            .headers()
            .get("cache-control")
            .map(|v| v.to_str().unwrap()),
        Some(IMMUTABLE_CACHE_CONTROL),
        "A 304 revalidation must still carry the immutable Cache-Control directive"
    );
}

#[tokio::test]
async fn html_shell_keeps_its_own_distinct_cache_control() {
    let test_app = spawn_app().await;

    let response = test_app
        .api_client
        .get(format!("{}/", test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status().as_u16(), 200);

    let cache_control = response
        .headers()
        .get("cache-control")
        .expect("HTML shell should set a Cache-Control header")
        .to_str()
        .unwrap();

    assert_ne!(
        cache_control, IMMUTABLE_CACHE_CONTROL,
        "The HTML shell must revalidate, not be cached forever like hashed assets"
    );
    assert_eq!(
        cache_control, "max-age=604800, must-revalidate",
        "The HTML shell's revalidation policy is deliberate and out of scope for asset caching changes"
    );
}

#[tokio::test]
async fn unknown_path_under_assets_still_returns_404() {
    let test_app = spawn_app().await;

    let response = test_app
        .api_client
        .get(format!("{}/assets/does-not-exist.js", test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(
        response.status().as_u16(),
        404,
        "A missing file under /assets should still 404, not fall through to the SPA shell"
    );

    let cache_control = response.headers().get("cache-control");
    assert_ne!(
        cache_control.map(|v| v.to_str().unwrap()),
        Some(IMMUTABLE_CACHE_CONTROL),
        "A 404 for a missing asset must not carry the immutable Cache-Control directive, or a \
         browser/CDN could cache the miss for a full year"
    );
}
