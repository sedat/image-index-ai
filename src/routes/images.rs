use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};
use tokio::time::timeout;

use crate::errors::{AppError, AppResult};
use crate::models::Photo;
use crate::state::AppState;
use crate::storage::{decode_image, infer_mime_type, remove_image, sanitize_file_name, save_image};
use crate::tagging::parse_tags;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/images", get(list_images).post(upload_image))
        .route("/api/images/search", post(search_images))
        .route("/api/images/semantic-search", post(semantic_search_images))
}

async fn upload_image(
    State(state): State<AppState>,
    Json(payload): Json<UploadRequest>,
) -> AppResult<Json<UploadResponse>> {
    if payload.file_name.trim().is_empty() {
        warn!("upload rejected: empty file_name");
        return Err(AppError::bad_request("file_name cannot be empty"));
    }

    if payload.image_base64.trim().is_empty() {
        warn!("upload rejected: empty image_base64");
        return Err(AppError::bad_request("image_base64 cannot be empty"));
    }

    info!(
        file_name = payload.file_name.as_str(),
        "received upload request"
    );

    let sanitized_name = sanitize_file_name(&payload.file_name)?;
    debug!(
        original = payload.file_name.as_str(),
        sanitized = sanitized_name.as_str(),
        "sanitized upload file name"
    );

    let image_bytes = decode_image(&payload.image_base64)?;
    info!(
        file_name = sanitized_name.as_str(),
        byte_len = image_bytes.len(),
        "decoded upload payload"
    );

    let mime_type = payload
        .mime_type
        .as_deref()
        .or_else(|| infer_mime_type(&sanitized_name))
        .ok_or_else(|| AppError::bad_request("unknown file extension; provide mime_type"))?;

    let canonical_base64 = STANDARD.encode(&image_bytes);

    debug!(
        file_name = sanitized_name.as_str(),
        mime_type, "requesting tags from LM Studio"
    );
    let tags = state
        .lm_client
        .tag_image(&canonical_base64, mime_type)
        .await
        .map_err(AppError::from)?;

    if tags.is_empty() {
        warn!(
            file_name = sanitized_name.as_str(),
            "tagging service returned no tags"
        );
        return Err(AppError::bad_request("tagging service returned no tags"));
    }

    info!(
        file_name = sanitized_name.as_str(),
        tag_count = tags.len(),
        "received tags from LM Studio"
    );

    let saved_path = save_image(&sanitized_name, &image_bytes).await?;
    info!(
        file_name = sanitized_name.as_str(),
        path = saved_path.as_str(),
        "saved image to disk"
    );

    // Compute a text embedding over the generated tags for semantic search
    let tag_text = tags.join(", ");
    let tag_embedding = match timeout(
        Duration::from_secs(5),
        state.lm_client.embed_texts(&vec![tag_text.clone()]),
    )
    .await
    {
        Ok(Ok(emb)) => emb.into_iter().next().map(pgvector::Vector::from),
        Ok(Err(err)) => {
            warn!(error = ?err, "embedding service failed; proceeding without vector");
            None
        }
        Err(_) => {
            warn!("embedding service timed out; proceeding without vector");
            None
        }
    };

    let photo = match Photo::add_photo(
        &state.pool,
        &sanitized_name,
        &saved_path,
        &tags,
        tag_embedding.as_ref(),
    )
    .await
    {
        Ok(photo) => photo,
        Err(err) => {
            remove_image(&saved_path).await;
            return Err(AppError::from(err));
        }
    };

    info!(
        photo_id = photo.photo_id,
        file_name = photo.file_name.as_str(),
        "persisted photo metadata"
    );

    Ok(Json(UploadResponse { photo }))
}

async fn list_images(
    State(state): State<AppState>,
    Query(params): Query<ListImagesParams>,
) -> AppResult<Json<PhotosResponse>> {
    let photos = if let Some(tags_param) = params.tags {
        let tags = parse_tags(&tags_param);
        if tags.is_empty() {
            let photos = Photo::list_all(&state.pool).await.map_err(AppError::from)?;
            info!(
                filter = "tags",
                requested = tags_param.as_str(),
                result_count = photos.len(),
                "parsed no tags; returning all photos"
            );
            photos
        } else {
            let photos = Photo::search_by_tags(&state.pool, &tags)
                .await
                .map_err(AppError::from)?;
            info!(
                filter = "tags",
                tag_count = tags.len(),
                result_count = photos.len(),
                "returning photos matching tags"
            );
            photos
        }
    } else {
        let photos = Photo::list_all(&state.pool).await.map_err(AppError::from)?;
        info!(
            result_count = photos.len(),
            "returning all photos without filters"
        );
        photos
    };

    Ok(Json(PhotosResponse { photos }))
}

async fn search_images(
    State(state): State<AppState>,
    Json(body): Json<SearchRequest>,
) -> AppResult<Json<SearchResponse>> {
    if body.query.trim().is_empty() {
        warn!("search rejected: empty query");
        return Err(AppError::bad_request("query cannot be empty"));
    }

    let trimmed_query = body.query.trim();

    info!(query = trimmed_query, "received semantic search request");

    let tags = state
        .lm_client
        .tags_from_query(trimmed_query)
        .await
        .map_err(AppError::from)?;

    let photos = if tags.is_empty() {
        Vec::new()
    } else {
        Photo::search_by_tags(&state.pool, &tags)
            .await
            .map_err(AppError::from)?
    };

    info!(
        query = trimmed_query,
        tag_count = tags.len(),
        result_count = photos.len(),
        "completed semantic search"
    );

    Ok(Json(SearchResponse {
        query: body.query,
        tags,
        photos,
    }))
}

async fn semantic_search_images(
    State(state): State<AppState>,
    Json(body): Json<VectorSearchRequest>,
) -> AppResult<Json<VectorSearchResponse>> {
    let trimmed_query = body.query.trim();
    if trimmed_query.is_empty() {
        warn!("semantic search rejected: empty query");
        return Err(AppError::bad_request("query cannot be empty"));
    }

    let requested_limit = body.limit.unwrap_or(24);
    let inputs = vec![trimmed_query.to_string()];

    info!(
        query = trimmed_query,
        limit = requested_limit,
        max_distance = body.max_distance,
        "received vector search request"
    );

    // 1) Try embeddings with a short timeout
    let mut fallback_reason: Option<String> = None;
    let embeddings = match timeout(Duration::from_secs(5), state.lm_client.embed_texts(&inputs)).await {
        Ok(Ok(v)) => {
            info!(
                query = trimmed_query,
                vector_count = v.len(),
                vector_dims = v.first().map(|vec| vec.len()),
                "embedding request succeeded"
            );
            if v.is_empty() {
                fallback_reason = Some("embedding service returned no vectors".to_string());
            }
            v
        }
        Ok(Err(err)) => {
            fallback_reason = Some("embedding request failed".to_string());
            warn!(error = ?err, "embedding request failed; attempting tag fallback");
            Vec::new()
        }
        Err(_) => {
            fallback_reason = Some("embedding request timed out".to_string());
            warn!("embedding request timed out; attempting tag fallback");
            Vec::new()
        }
    };

    // If we got an embedding, attempt ANN search first
    if let Some(first) = embeddings.into_iter().next() {
        let query_vec = pgvector::Vector::from(first);
        let mut limit = requested_limit;
        if limit < 1 {
            limit = 1;
        } else if limit > 200 {
            limit = 200;
        }

        // If client supplies a max_distance use it; otherwise use adaptive threshold inside the query
        let max_distance = body.max_distance;

        let photos = Photo::search_by_embedding(&state.pool, &query_vec, limit, max_distance)
            .await
            .map_err(AppError::from)?;

        if !photos.is_empty() {
            info!(
                query = trimmed_query,
                result_count = photos.len(),
                limit_applied = limit,
                max_distance = max_distance,
                "vector search succeeded"
            );
            return Ok(Json(VectorSearchResponse { query: body.query, photos, tags: None }));
        }
        fallback_reason
            .get_or_insert_with(|| "vector search returned no results".to_string());
        info!(
            query = trimmed_query,
            limit_applied = limit,
            max_distance = max_distance,
            fallback_reason = fallback_reason.as_deref(),
            "vector search returned no results; proceeding to tag fallback"
        );
    }

    // 2) Tag fallback with tight timeout
    let fallback_tags = match timeout(Duration::from_secs(2), state.lm_client.tags_from_query(trimmed_query)).await {
        Ok(Ok(tags)) => {
            info!(
                query = trimmed_query,
                tag_count = tags.len(),
                tags = ?tags,
                "fallback tag extraction succeeded"
            );
            if tags.is_empty() {
                fallback_reason
                    .get_or_insert_with(|| "fallback tagging returned no tags".to_string());
            }
            Some(tags)
        }
        Ok(Err(err)) => {
            fallback_reason
                .get_or_insert_with(|| "fallback tagging failed".to_string());
            warn!(error = ?err, "tag extraction failed during fallback");
            None
        }
        Err(_) => {
            fallback_reason
                .get_or_insert_with(|| "fallback tagging timed out".to_string());
            warn!("tag extraction timed out during fallback");
            None
        }
    };

    let photos = if let Some(tags) = &fallback_tags {
        if tags.is_empty() {
            Vec::new()
        } else {
            Photo::search_by_tags(&state.pool, tags)
                .await
                .map_err(AppError::from)?
        }
    } else {
        Vec::new()
    };

    info!(
        query = trimmed_query,
        result_count = photos.len(),
        fallback_reason = fallback_reason.as_deref(),
        fallback_tags_count = fallback_tags.as_ref().map(|tags| tags.len()),
        fallback_tags = ?fallback_tags,
        "completed semantic search"
    );

    Ok(Json(VectorSearchResponse { query: body.query, photos, tags: fallback_tags }))
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

#[derive(Debug, Deserialize)]
struct VectorSearchRequest {
    query: String,
    limit: Option<i64>,
    max_distance: Option<f32>,
}

#[derive(Debug, Serialize)]
struct VectorSearchResponse {
    query: String,
    photos: Vec<Photo>,
    tags: Option<Vec<String>>,
}
