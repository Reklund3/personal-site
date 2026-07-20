use crate::helpers::spawn_app;

#[tokio::test]
async fn resume_download_requires_revalidation() {
    let app = spawn_app().await;

    let response = app.get_resume().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response.headers().get("cache-control").unwrap(),
        "no-cache, must-revalidate"
    );
    assert!(response.headers().get("etag").is_some());
    let disposition = response
        .headers()
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap();
    assert!(disposition.contains("attachment"));
}

#[tokio::test]
async fn resume_conditional_request_returns_304_when_unchanged() {
    let app = spawn_app().await;

    let first = app.get_resume().await;
    let etag = first
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let response = app.get_resume_conditional(&etag).await;

    assert_eq!(response.status().as_u16(), 304);
    assert_eq!(
        response.headers().get("cache-control").unwrap(),
        "no-cache, must-revalidate"
    );
}

#[tokio::test]
async fn resume_is_served_fresh_after_the_file_changes() {
    let app = spawn_app().await;

    let first = app.get_resume().await;
    let old_etag = first
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    std::fs::write(
        &app.resume_path,
        b"%PDF-1.4\nupdated resume with different contents\n",
    )
    .expect("Failed to update test resume.");

    let response = app.get_resume_conditional(&old_etag).await;

    assert_eq!(response.status().as_u16(), 200);
    let new_etag = response.headers().get("etag").unwrap().to_str().unwrap();
    assert_ne!(new_etag, old_etag);
}

#[tokio::test]
async fn headshot_requires_revalidation() {
    let app = spawn_app().await;

    let response = app.get_headshot().await;

    assert_eq!(response.status().as_u16(), 200);
    assert_eq!(
        response.headers().get("cache-control").unwrap(),
        "no-cache, must-revalidate"
    );
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "image/jpeg"
    );
    assert!(response.headers().get("etag").is_some());
}

#[tokio::test]
async fn headshot_conditional_request_returns_304_when_unchanged() {
    let app = spawn_app().await;

    let first = app.get_headshot().await;
    let etag = first
        .headers()
        .get("etag")
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();

    let response = app.get_headshot_conditional(&etag).await;

    assert_eq!(response.status().as_u16(), 304);
    assert_eq!(
        response.headers().get("cache-control").unwrap(),
        "no-cache, must-revalidate"
    );
}
