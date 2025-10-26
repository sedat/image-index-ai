use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};
use crate::models::Photo;
use crate::state::AppState;
use crate::storage::{decode_image, infer_mime_type, remove_image, sanitize_file_name, save_image};
use crate::tagging::parse_tags;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/images", get(list_images).post(upload_image))
        .route("/api/images/search", post(search_images))
}

async fn upload_image(
    State(state): State<AppState>,
    Json(payload): Json<UploadRequest>,
) -> AppResult<Json<UploadResponse>> {
    if payload.file_name.trim().is_empty() {
        return Err(AppError::bad_request("file_name cannot be empty"));
    }

    if payload.image_base64.trim().is_empty() {
        return Err(AppError::bad_request("image_base64 cannot be empty"));
    }

    let sanitized_name = sanitize_file_name(&payload.file_name)?;
    let image_bytes = decode_image(&payload.image_base64)?;

    let mime_type = payload
        .mime_type
        .as_deref()
        .or_else(|| infer_mime_type(&sanitized_name))
        .ok_or_else(|| AppError::bad_request("unknown file extension; provide mime_type"))?;

    let canonical_base64 = STANDARD.encode(&image_bytes);

    let tags = state
        .lm_client
        .tag_image(&canonical_base64, mime_type)
        .await
        .map_err(AppError::from)?;

    if tags.is_empty() {
        return Err(AppError::bad_request("tagging service returned no tags"));
    }

    let saved_path = save_image(&sanitized_name, &image_bytes).await?;

    let photo = match Photo::add_photo(&state.pool, &sanitized_name, &saved_path, &tags).await {
        Ok(photo) => photo,
        Err(err) => {
            remove_image(&saved_path).await;
            return Err(AppError::from(err));
        }
    };

    Ok(Json(UploadResponse { photo }))
}

async fn list_images(
    State(state): State<AppState>,
    Query(params): Query<ListImagesParams>,
) -> AppResult<Json<PhotosResponse>> {
    let photos = if let Some(tags_param) = params.tags {
        let tags = parse_tags(&tags_param);
        if tags.is_empty() {
            Photo::list_all(&state.pool).await.map_err(AppError::from)?
        } else {
            Photo::search_by_tags(&state.pool, &tags)
                .await
                .map_err(AppError::from)?
        }
    } else {
        Photo::list_all(&state.pool).await.map_err(AppError::from)?
    };

    Ok(Json(PhotosResponse { photos }))
}

async fn search_images(
    State(state): State<AppState>,
    Json(body): Json<SearchRequest>,
) -> AppResult<Json<SearchResponse>> {
    if body.query.trim().is_empty() {
        return Err(AppError::bad_request("query cannot be empty"));
    }

    let tags = state
        .lm_client
        .tags_from_query(body.query.trim())
        .await
        .map_err(AppError::from)?;

    let photos = if tags.is_empty() {
        Vec::new()
    } else {
        Photo::search_by_tags(&state.pool, &tags)
            .await
            .map_err(AppError::from)?
    };

    Ok(Json(SearchResponse {
        query: body.query,
        tags,
        photos,
    }))
}

#[derive(Debug, Deserialize)]
struct UploadRequest {
    file_name: String,
    image_base64: String,
    mime_type: Option<String>,
}

#[derive(Debug, Serialize)]
struct UploadResponse {
    photo: Photo,
}

#[derive(Debug, Deserialize)]
struct ListImagesParams {
    tags: Option<String>,
}

#[derive(Debug, Serialize)]
struct PhotosResponse {
    photos: Vec<Photo>,
}

#[derive(Debug, Deserialize)]
struct SearchRequest {
    query: String,
}

#[derive(Debug, Serialize)]
struct SearchResponse {
    query: String,
    tags: Vec<String>,
    photos: Vec<Photo>,
}
