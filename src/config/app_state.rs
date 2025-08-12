// src/app_state.rs
use crate::config::ServerConfig;
use sqlx::PgPool;
use std::time::Instant;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cfg: ServerConfig,
    #[allow(dead_code)]
    pub started_at: Instant,
}

impl AppState {
    pub fn new(pool: PgPool, cfg: ServerConfig) -> Self {
        Self {
            pool,
            cfg,
            started_at: Instant::now(),
        }
    }
}
