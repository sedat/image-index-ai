use chrono::NaiveDateTime;
use pgvector::Vector;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Photo {
    pub photo_id: i32,
    pub file_name: String,
    pub file_path: String,
    pub tags: Vec<String>,
    pub created_at: NaiveDateTime,
}

impl Photo {
    pub async fn add_photo(
        pool: &PgPool,
        file_name: &str,
        file_path: &str,
        tags: &[String],
        tag_embedding: Option<&Vector>,
    ) -> Result<Photo, sqlx::Error> {
        let tags_vec = tags.to_vec();
        let tag_emb_param: Option<Vector> = tag_embedding.cloned();

        sqlx::query_as::<_, Photo>(
            "INSERT INTO photos (file_name, file_path, tags, tag_embedding) VALUES ($1, $2, $3, $4) RETURNING photo_id, file_name, file_path, tags, created_at",
        )
        .bind(file_name)
        .bind(file_path)
        .bind(tags_vec)
        .bind(tag_emb_param)
        .fetch_one(pool)
        .await
    }

    pub async fn list_all(pool: &PgPool) -> Result<Vec<Photo>, sqlx::Error> {
        sqlx::query_as::<_, Photo>(
            "SELECT photo_id, file_name, file_path, tags, created_at FROM photos ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn search_by_tags(
        pool: &PgPool,
        search_tags: &[String],
    ) -> Result<Vec<Photo>, sqlx::Error> {
        if search_tags.is_empty() {
            return Self::list_all(pool).await;
        }

        let tags: Vec<&str> = search_tags.iter().map(String::as_str).collect();

        sqlx::query_as::<_, Photo>(
            "SELECT photo_id, file_name, file_path, tags, created_at FROM photos WHERE tags && $1::text[] ORDER BY created_at DESC",
        )
        .bind(tags)
        .fetch_all(pool)
        .await
    }

    pub async fn search_by_embedding(
        pool: &PgPool,
        query_embedding: &Vector,
        limit: i64,
        max_distance: Option<f32>,
    ) -> Result<Vec<Photo>, sqlx::Error> {
        let emb: Vector = query_embedding.clone();
        let mut tx = pool.begin().await?;
        // Prefer HNSW if present; also set ivfflat probes as a no-op safeguard.
        sqlx::query("SET LOCAL hnsw.ef_search = 80")
            .execute(&mut *tx)
            .await?;
        sqlx::query("SET LOCAL ivfflat.probes = 100")
            .execute(&mut *tx)
            .await?;

        let rows = if let Some(threshold) = max_distance {
            sqlx::query_as::<_, Photo>(
                "SELECT photo_id, file_name, file_path, tags, created_at \
                 FROM photos \
                 WHERE tag_embedding IS NOT NULL \
                   AND (tag_embedding <=> $1) <= $3 \
                 ORDER BY tag_embedding <=> $1 \
                 LIMIT $2",
            )
            .bind(emb)
            .bind(limit)
            .bind(threshold)
            .fetch_all(&mut *tx)
            .await?
        } else {
            // Adaptive threshold: keep rows within min_dist + delta, capped by max_cap
            // This trims broad matches while maintaining nearest neighbors.
            let delta: f32 = 0.05;
            let max_cap: f32 = 0.60;
            sqlx::query_as::<_, Photo>(
                "WITH ranked AS (
                    SELECT photo_id, file_name, file_path, tags, created_at,
                           (tag_embedding <=> $1) AS dist
                    FROM photos
                    WHERE tag_embedding IS NOT NULL
                    ORDER BY dist
                    LIMIT $2
                 )
                 SELECT photo_id, file_name, file_path, tags, created_at
                 FROM ranked
                 WHERE dist <= LEAST((SELECT MIN(dist) FROM ranked) + $3, $4)
                 ORDER BY dist",
            )
            .bind(emb)
            .bind(limit)
            .bind(delta)
            .bind(max_cap)
            .fetch_all(&mut *tx)
            .await?
        };

        tx.commit().await?;
        Ok(rows)
    }
}
