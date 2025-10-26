use chrono::NaiveDateTime;
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
    ) -> Result<Photo, sqlx::Error> {
        let tags_vec = tags.to_vec();

        sqlx::query_as::<_, Photo>(
            "INSERT INTO photos (file_name, file_path, tags) VALUES ($1, $2, $3) RETURNING photo_id, file_name, file_path, tags, created_at",
        )
        .bind(file_name)
        .bind(file_path)
        .bind(tags_vec)
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
}
