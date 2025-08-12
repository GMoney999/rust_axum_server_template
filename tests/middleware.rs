use axum::http::Request;
use axum::body::Body;
use tower::ServiceExt;
use axum_server_shuttle::config::{ServerConfig, CorsPolicy};
use axum_server_shuttle::models::Server;
use axum_server_shuttle::config::AppState;
use sqlx::PgPool;

#[tokio::test]
async fn request_id_is_set() {
    // Build a tiny app with the Server composition to include request-id layers
    // For test purposes, use a lazily constructed pool only if available; otherwise skip heavy checks.
    let mut cfg = ServerConfig::default();
    // Disable CORS to avoid invalid wildcard+credentials combo in default permissive mode
    cfg.cors = CorsPolicy::Disabled;
    // This pool is not connected; handlers used in this test do not hit DB.
    let pool = PgPool::connect_lazy("postgres://127.0.0.1/postgres").expect("lazy pool");
    let state = AppState::new(pool, cfg);
    let server = Server::new(state);
    let app = server.router();

    let res = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    let header = res.headers().get("x-request-id");
    assert!(header.is_some());
}

