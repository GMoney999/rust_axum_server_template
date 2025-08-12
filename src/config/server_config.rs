// src/server_config.rs
use anyhow::{Context, Result};
use axum::http::{HeaderName, HeaderValue};
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub request_id_header: HeaderName, // e.g., "x-request-id"
    pub timeout: Duration,             // global handler timeout
    pub cors: CorsPolicy,              // tiny switch for your use case
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            request_id_header: HeaderName::from_static("x-request-id"),
            timeout: Duration::from_secs(15),
            cors: CorsPolicy::Permissive,
        }
    }
}

impl ServerConfig {
    /// Minimal loader: environment variables are optional.
    /// - REQUEST_ID_HEADER (default: x-request-id)
    /// - TIMEOUT_SECS      (default: 15)
    /// - CORS_ALLOWED_ORIGINS: comma-separated list -> Allow([...])
    /// - CORS_DISABLED: any non-empty value -> Disabled
    pub fn load_from_env() -> Result<Self> {
        use std::env;

        let mut cfg = ServerConfig::default();

        if let Ok(h) = env::var("REQUEST_ID_HEADER") {
            cfg.request_id_header =
                HeaderName::from_lowercase(h.as_bytes()).context("invalid REQUEST_ID_HEADER")?;
        }

        if let Ok(secs) = env::var("TIMEOUT_SECS") {
            let s: u64 = secs.parse().context("TIMEOUT_SECS must be u64")?;
            cfg.timeout = Duration::from_secs(s);
        }

        // CORS precedence: disabled > allow-list > permissive(default)
        if env::var("CORS_DISABLED").is_ok() {
            cfg.cors = CorsPolicy::Disabled;
        } else if let Ok(csv) = env::var("CORS_ALLOWED_ORIGINS") {
            let mut origins = Vec::new();
            for o in csv.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                origins.push(
                    HeaderValue::from_str(o).context("invalid origin in CORS_ALLOWED_ORIGINS")?,
                );
            }
            if !origins.is_empty() {
                cfg.cors = CorsPolicy::Allow(origins);
            }
        }

        Ok(cfg)
    }
}

#[derive(Clone, Debug)]
pub enum CorsPolicy {
    /// Allow everything (great for local tools, personal dashboards).
    Permissive,
    /// Only allow these exact origins.
    Allow(Vec<HeaderValue>),
    /// No CORS headers at all.
    Disabled,
}
