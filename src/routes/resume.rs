use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Result, web};
use std::path::PathBuf;

#[derive(Clone)]
pub struct ResumeConfig {
    pub file_path: String,
}

#[tracing::instrument(name = "Serving resume", skip(config, req))]
pub async fn serve_resume(
    config: web::Data<ResumeConfig>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let path = PathBuf::from(&config.file_path);

    // Validate file exists
    if !path.exists() {
        tracing::warn!("Resume file not found at {:?}", path);
        return Err(actix_web::error::ErrorNotFound("Resume not available"));
    }

    // Validate file size (prevent serving huge files - max 10MB)
    let metadata = std::fs::metadata(&path).map_err(|e| {
        tracing::error!("Failed to read resume metadata: {}", e);
        actix_web::error::ErrorNotFound("Resume not available")
    })?;

    if metadata.len() > 10 * 1024 * 1024 {
        tracing::error!("Resume file too large: {} bytes", metadata.len());
        return Err(actix_web::error::ErrorNotFound("Resume not available"));
    }

    // Verify it's a PDF by checking magic bytes
    let mut file = std::fs::File::open(&path).map_err(|e| {
        tracing::error!("Failed to open resume file: {}", e);
        actix_web::error::ErrorNotFound("Resume not available")
    })?;

    let mut magic = [0u8; 4];
    use std::io::Read;
    file.read_exact(&mut magic).map_err(|e| {
        tracing::error!("Failed to read PDF magic bytes: {}", e);
        actix_web::error::ErrorNotFound("Resume not available")
    })?;

    if &magic != b"%PDF" {
        tracing::error!("Invalid PDF file at {:?}", path);
        return Err(actix_web::error::ErrorNotFound("Resume not available"));
    }

    tracing::info!("Serving resume from {:?}", path);

    // NamedFile generates the ETag/Last-Modified headers and answers conditional
    // requests (If-None-Match / If-Modified-Since) with 304s in into_response.
    let named_file = NamedFile::open(path)?
        .use_etag(true)
        .use_last_modified(true)
        .set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Attachment,
            parameters: vec![actix_web::http::header::DispositionParam::Filename(
                "Robert_Eklund_Resume.pdf".to_string(),
            )],
        });

    let mut response = named_file.into_response(&req);

    // Always require revalidation so an updated resume is never served stale from cache.
    response.headers_mut().insert(
        actix_web::http::header::CACHE_CONTROL,
        actix_web::http::header::HeaderValue::from_static("no-cache, must-revalidate"),
    );

    Ok(response)
}
