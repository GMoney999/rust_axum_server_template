// src/main.rs
mod config;

mod middleware;
mod routes;
mod models;

use anyhow::Context;
use config::{AppState, ServerConfig};
use models::Server;

use shuttle_axum::ShuttleAxum;
use shuttle_runtime::SecretStore;
use sqlx::PgPool;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[shuttle_runtime::main]
async fn axum(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleAxum {
    // Tracing: `RUST_LOG=tower_http=info,axum_server_shuttle=debug` etc.
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "info,tower_http=info,sqlx=warn".into()))
        .init();

    // Run database migrations at startup
    sqlx::migrate!().run(&pool).await.expect("Failed to run Migrations :(");

    let cfg = ServerConfig::load_from_env().expect("config");
    let state = AppState::new(pool, cfg);
    let server = Server::new(state);

    let mut app = server.router();

    if let Some(admin) = secrets.get("ADMIN_TOKEN") {
        // Lock down everything behind a static bearer token
        // Authorization: Bearer <ADMIN_TOKEN>
        let layer = ValidateRequestHeaderLayer::bearer(admin.as_str());
        app = app.layer(layer);
    }

    Ok(app.into())
}