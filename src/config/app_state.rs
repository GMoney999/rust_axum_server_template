// src/app_state.rs
use std::time::Instant;
use sqlx::PgPool;
use crate::config::ServerConfig;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cfg: ServerConfig,
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