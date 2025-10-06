use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

/// Serve robots.txt file
#[tracing::instrument(name = "Serving robots.txt")]
pub async fn serve_robots_txt(_req: HttpRequest) -> Result<NamedFile> {
    let path = PathBuf::from("./public/robots.txt");

    if !path.exists() {
        tracing::error!("robots.txt not found at {:?}", path);
        return Err(actix_web::error::ErrorNotFound("robots.txt not found"));
    }

    tracing::debug!("Serving robots.txt from {:?}", path);

    Ok(NamedFile::open(path)?
        .use_last_modified(true)
        .set_content_type("text/plain; charset=utf-8".parse().unwrap()))
}

/// Serve sitemap.xml file
#[tracing::instrument(name = "Serving sitemap.xml")]
pub async fn serve_sitemap_xml(_req: HttpRequest) -> Result<NamedFile> {
    let path = PathBuf::from("./public/sitemap.xml");

    if !path.exists() {
        tracing::error!("sitemap.xml not found at {:?}", path);
        return Err(actix_web::error::ErrorNotFound("sitemap.xml not found"));
    }

    tracing::debug!("Serving sitemap.xml from {:?}", path);

    Ok(NamedFile::open(path)?
        .use_last_modified(true)
        .set_content_type("application/xml; charset=utf-8".parse().unwrap()))
}

/// Serve ai.txt file for AI crawler attribution
#[tracing::instrument(name = "Serving ai.txt")]
pub async fn serve_ai_txt(_req: HttpRequest) -> Result<NamedFile> {
    let path = PathBuf::from("./public/ai.txt");

    if !path.exists() {
        tracing::error!("ai.txt not found at {:?}", path);
        return Err(actix_web::error::ErrorNotFound("ai.txt not found"));
    }

    tracing::debug!("Serving ai.txt from {:?}", path);

    Ok(NamedFile::open(path)?
        .use_last_modified(true)
        .set_content_type("text/plain; charset=utf-8".parse().unwrap()))
}
