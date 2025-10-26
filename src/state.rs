use sqlx::PgPool;

use crate::services::LmStudioClient;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub lm_client: LmStudioClient,
}
