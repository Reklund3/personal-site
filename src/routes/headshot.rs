use actix_files::NamedFile;
use actix_web::{HttpRequest, HttpResponse, Result, web};
use std::io::Read;
use std::path::PathBuf;

#[derive(Clone)]
pub struct HeadshotConfig {
    pub file_path: String,
}

#[derive(Debug, PartialEq)]
enum ImageFormat {
    Jpeg,
    Png,
    Webp,
}

impl ImageFormat {
    fn content_type(&self) -> &'static str {
        match self {
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Png => "image/png",
            ImageFormat::Webp => "image/webp",
        }
    }

    fn from_magic_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }

        // JPEG: FF D8 FF
        if bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
            return Some(ImageFormat::Jpeg);
        }

        // PNG: 89 50 4E 47 0D 0A 1A 0A
        if bytes.len() >= 8
            && bytes[0] == 0x89
            && bytes[1] == 0x50
            && bytes[2] == 0x4E
            && bytes[3] == 0x47
        {
            return Some(ImageFormat::Png);
        }

        // WebP: RIFF....WEBP (52 49 46 46 ... 57 45 42 50)
        if bytes.len() >= 12
            && bytes[0] == 0x52
            && bytes[1] == 0x49
            && bytes[2] == 0x46
            && bytes[3] == 0x46
            && bytes[8] == 0x57
            && bytes[9] == 0x45
            && bytes[10] == 0x42
            && bytes[11] == 0x50
        {
            return Some(ImageFormat::Webp);
        }

        None
    }
}

#[tracing::instrument(name = "Serving headshot", skip(config, req))]
pub async fn serve_headshot(
    config: web::Data<HeadshotConfig>,
    req: HttpRequest,
) -> Result<HttpResponse> {
    let path = PathBuf::from(&config.file_path);

    // Validate file exists
    if !path.exists() {
        tracing::warn!("Headshot file not found at {:?}", path);
        return Err(actix_web::error::ErrorNotFound("Headshot not available"));
    }

    // Validate file size (max 5MB for images)
    let metadata = std::fs::metadata(&path).map_err(|e| {
        tracing::error!("Failed to read headshot metadata: {}", e);
        actix_web::error::ErrorNotFound("Headshot not available")
    })?;

    if metadata.len() > 5 * 1024 * 1024 {
        tracing::error!("Headshot file too large: {} bytes", metadata.len());
        return Err(actix_web::error::ErrorNotFound("Headshot not available"));
    }

    // Read magic bytes to detect image format
    let mut file = std::fs::File::open(&path).map_err(|e| {
        tracing::error!("Failed to open headshot file: {}", e);
        actix_web::error::ErrorNotFound("Headshot not available")
    })?;

    let mut magic = [0u8; 12];
    let bytes_read = file.read(&mut magic).map_err(|e| {
        tracing::error!("Failed to read image magic bytes: {}", e);
        actix_web::error::ErrorNotFound("Headshot not available")
    })?;

    let image_format = ImageFormat::from_magic_bytes(&magic[..bytes_read]).ok_or_else(|| {
        tracing::error!("Invalid image format at {:?}", path);
        actix_web::error::ErrorNotFound("Headshot not available")
    })?;

    tracing::info!(
        "Serving headshot from {:?} (format: {:?}, size: {} bytes)",
        path,
        image_format,
        metadata.len()
    );

    // NamedFile generates the ETag/Last-Modified headers and answers conditional
    // requests (If-None-Match / If-Modified-Since) with 304s in into_response.
    let named_file = NamedFile::open(path)?
        .use_etag(true)
        .use_last_modified(true)
        .set_content_type(image_format.content_type().parse().unwrap());

    let mut response = named_file.into_response(&req);

    // Always require revalidation so an updated headshot is never served stale from cache.
    response.headers_mut().insert(
        actix_web::http::header::CACHE_CONTROL,
        actix_web::http::header::HeaderValue::from_static("no-cache, must-revalidate"),
    );

    Ok(response)
}
