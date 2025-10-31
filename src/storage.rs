use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::Context;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use tokio::fs;
use tracing::{info, warn};

use crate::errors::{AppError, AppResult};

const IMAGES_DIR: &str = "images";

pub fn sanitize_file_name(file_name: &str) -> AppResult<String> {
    let trimmed = file_name.trim();
    let candidate = Path::new(trimmed)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::bad_request("file_name must not contain path separators"))?;

    if candidate.is_empty() {
        return Err(AppError::bad_request("file_name cannot be empty"));
    }

    if candidate.contains(' ') {
        return Ok(candidate.replace(' ', "_"));
    }

    Ok(candidate.to_string())
}

pub fn decode_image(encoded: &str) -> AppResult<Vec<u8>> {
    let cleaned = encoded.replace(['\n', '\r'], "");
    STANDARD
        .decode(cleaned.as_bytes())
        .map_err(|_| AppError::bad_request("image_base64 must be valid base64"))
}

pub fn infer_mime_type(file_name: &str) -> Option<&'static str> {
    let ext = Path::new(file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase());

    match ext.as_deref() {
        Some("png") => Some("image/png"),
        Some("jpg") | Some("jpeg") => Some("image/jpeg"),
        Some("gif") => Some("image/gif"),
        Some("bmp") => Some("image/bmp"),
        _ => None,
    }
}

pub async fn save_image(file_name: &str, bytes: &[u8]) -> AppResult<String> {
    let dir = Path::new(IMAGES_DIR);
    fs::create_dir_all(dir)
        .await
        .context("failed to ensure images directory exists")
        .map_err(AppError::from)?;

    let path: PathBuf = dir.join(file_name);

    fs::write(&path, bytes)
        .await
        .with_context(|| format!("failed to write {}", path.display()))
        .map_err(AppError::from)?;

    info!(
        path = %path.display(),
        byte_len = bytes.len(),
        "saved image bytes to disk"
    );

    path.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| AppError::internal("stored path is not valid UTF-8"))
}

pub async fn remove_image(path: &str) {
    if path.is_empty() {
        return;
    }

    if let Err(err) = fs::remove_file(path).await {
        if err.kind() != ErrorKind::NotFound {
            warn!(file = path, error = ?err, "failed to remove orphaned file");
        }
    }
}
