use anyhow::{Context, Result};
use sqlx::PgPool;
use tracing::{debug, info};

const MIGRATIONS: &[Migration] = &[
    Migration {
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
    },
    Migration {
        id: "0002_enable_pgvector",
        sql: r#"
            CREATE EXTENSION IF NOT EXISTS vector;
        "#,
    },
    Migration {
        id: "0003_add_tag_embedding_column",
        sql: r#"
            -- Using cosine distance, pick a dimension matching your embed model
            ALTER TABLE photos ADD COLUMN IF NOT EXISTS tag_embedding vector(768);
        "#,
    },
    Migration {
        id: "0004_create_tag_embedding_index",
        sql: r#"
            -- Approximate index for faster ANN search; adjust lists to your data size
            DO $$ BEGIN
                IF NOT EXISTS (
                    SELECT 1 FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relname = 'idx_photos_tag_embedding' AND n.nspname = 'public'
                ) THEN
                    CREATE INDEX idx_photos_tag_embedding
                    ON photos USING ivfflat (tag_embedding vector_cosine_ops)
                    WITH (lists = 100);
                END IF;
            END $$;
        "#,
    },
    Migration {
        id: "0005_reset_tag_embedding_dimension",
        sql: r#"
            -- Ensure existing installs use the correct 768-dim column and index
            DO $$
            DECLARE
                current_dim INTEGER;
            BEGIN
                SELECT atttypmod
                INTO current_dim
                FROM pg_attribute
                WHERE attrelid = 'photos'::regclass
                  AND attname = 'tag_embedding'
                  AND attnum > 0
                  AND NOT attisdropped;

                IF current_dim IS NULL THEN
                    EXECUTE 'ALTER TABLE photos ADD COLUMN IF NOT EXISTS tag_embedding vector(768)';
                ELSIF current_dim <> 768 THEN
                    EXECUTE 'DROP INDEX IF EXISTS idx_photos_tag_embedding';
                    EXECUTE 'ALTER TABLE photos DROP COLUMN tag_embedding';
                    EXECUTE 'ALTER TABLE photos ADD COLUMN tag_embedding vector(768)';
                END IF;

                EXECUTE 'CREATE INDEX IF NOT EXISTS idx_photos_tag_embedding
                    ON photos USING ivfflat (tag_embedding vector_cosine_ops)
                    WITH (lists = 100)';
            END $$;
        "#,
    },
    Migration {
        id: "0006_use_hnsw_index_for_embeddings",
        sql: r#"
            -- Switch to HNSW for better recall/latency at scale
            DO $$
            BEGIN
                -- Drop old ivfflat index if it exists
                IF EXISTS (
                    SELECT 1 FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relname = 'idx_photos_tag_embedding' AND n.nspname = 'public'
                ) THEN
                    EXECUTE 'DROP INDEX idx_photos_tag_embedding';
                END IF;

                -- Create HNSW index if it doesn't already exist
                IF NOT EXISTS (
                    SELECT 1 FROM pg_class c
                    JOIN pg_namespace n ON n.oid = c.relnamespace
                    WHERE c.relname = 'idx_photos_tag_embedding_hnsw' AND n.nspname = 'public'
                ) THEN
                    EXECUTE 'CREATE INDEX idx_photos_tag_embedding_hnsw
                             ON photos USING hnsw (tag_embedding vector_cosine_ops)
                             WITH (m = 16, ef_construction = 200)';
                END IF;
            END $$;
        "#,
    },
];

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
            debug!(id = migration.id, "migration already applied; skipping");
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
