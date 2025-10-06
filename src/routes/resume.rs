use actix_files::NamedFile;
use actix_web::{web, HttpRequest, Result};
use std::path::PathBuf;

#[derive(Clone)]
pub struct ResumeConfig {
    pub file_path: String,
}

#[tracing::instrument(name = "Serving resume", skip(config))]
pub async fn serve_resume(
    config: web::Data<ResumeConfig>,
    _req: HttpRequest,
) -> Result<NamedFile> {
    let path = PathBuf::from(&config.file_path);

    // Validate file exists
    if !path.exists() {
        tracing::warn!("Resume file not found at {:?}", path);
        return Err(actix_web::error::ErrorNotFound("Resume not available"));
    }

    // Validate file size (prevent serving huge files - max 10MB)
    let metadata = std::fs::metadata(&path)
        .map_err(|e| {
            tracing::error!("Failed to read resume metadata: {}", e);
            actix_web::error::ErrorNotFound("Resume not available")
        })?;

    if metadata.len() > 10 * 1024 * 1024 {
        tracing::error!("Resume file too large: {} bytes", metadata.len());
        return Err(actix_web::error::ErrorNotFound("Resume not available"));
    }

    // Verify it's a PDF by checking magic bytes
    let mut file = std::fs::File::open(&path)
        .map_err(|e| {
            tracing::error!("Failed to open resume file: {}", e);
            actix_web::error::ErrorNotFound("Resume not available")
        })?;

    let mut magic = [0u8; 4];
    use std::io::Read;
    file.read_exact(&mut magic)
        .map_err(|e| {
            tracing::error!("Failed to read PDF magic bytes: {}", e);
            actix_web::error::ErrorNotFound("Resume not available")
        })?;

    if &magic != b"%PDF" {
        tracing::error!("Invalid PDF file at {:?}", path);
        return Err(actix_web::error::ErrorNotFound("Resume not available"));
    }

    tracing::info!("Serving resume from {:?}", path);

    // Serve the file with proper headers for download
    Ok(NamedFile::open(path)?
        .use_last_modified(true)
        .set_content_disposition(actix_web::http::header::ContentDisposition {
            disposition: actix_web::http::header::DispositionType::Attachment,
            parameters: vec![actix_web::http::header::DispositionParam::Filename(
                "Robert_Eklund_Resume.pdf".to_string(),
            )],
        }))
}
