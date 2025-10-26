use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::info;

const MIGRATIONS: &[Migration] = &[Migration {
    id: "0001_create_photos_table",
    sql: r#"
        CREATE TABLE IF NOT EXISTS photos (
            photo_id SERIAL PRIMARY KEY,
            file_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            tags TEXT[] NOT NULL DEFAULT '{}',
            created_at TIMESTAMP DEFAULT NOW()
        );
    "#,
}];

#[derive(Copy, Clone)]
struct Migration {
    id: &'static str,
    sql: &'static str,
}

pub async fn run(pool: &PgPool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS app_migrations (id TEXT PRIMARY KEY, applied_at TIMESTAMP DEFAULT NOW())",
    )
    .execute(pool)
    .await
    .context("failed to ensure migrations table exists")?;

    for migration in MIGRATIONS {
        let applied: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM app_migrations WHERE id = $1)")
                .bind(migration.id)
                .fetch_one(pool)
                .await
                .context("failed to check migration status")?;

        if applied {
            continue;
        }

        info!(id = migration.id, "applying migration");

        sqlx::query(migration.sql)
            .execute(pool)
            .await
            .with_context(|| format!("failed to run migration {}", migration.id))?;

        sqlx::query("INSERT INTO app_migrations (id) VALUES ($1)")
            .bind(migration.id)
            .execute(pool)
            .await
            .with_context(|| format!("failed to record migration {}", migration.id))?;
    }

    Ok(())
}
