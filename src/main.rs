mod errors;
mod migrations;
mod models;
mod routes;
mod services;
mod state;
mod storage;
mod tagging;

use std::env;

use anyhow::{Context, Result};
use axum::extract::DefaultBodyLimit;
use axum::Router;
use reqwest::Client;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::migrations::run as run_migrations;
use crate::routes::images;
use crate::services::LmStudioClient;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    // let database_url =
    //     env::var("DATABASE_URL").context("DATABASE_URL environment variable must be set")?;

    let pool = PgPool::connect("postgres://user:password@localhost/locatai")
        .await
        .context("failed to connect to the database")?;

    run_migrations(&pool).await?;

    let lm_client = LmStudioClient::new(Client::new());
    let state = AppState { pool, lm_client };

    let app = Router::new()
        .merge(images::router())
        .with_state(state)
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024));

    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = TcpListener::bind(&bind_addr)
        .await
        .with_context(|| format!("failed to bind to {bind_addr}"))?;

    println!("Server listening on {bind_addr}");

    axum::serve(listener, app.into_make_service())
        .await
        .context("server error")?;

    Ok(())
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .try_init();
}
