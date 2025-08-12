// src/models/server.rs (composition point)
use crate::{
    config::AppState,
    middleware::Middleware,
    routes::{health, create_todo, get_all_todos},
};
use crate::middleware::MiddlewareSuite;
use axum::{
    Router,
    routing::{get, post}
};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::request_id::{SetRequestIdLayer, PropagateRequestIdLayer, MakeRequestUuid};

#[derive(Clone)]
pub struct Server {
    pub state: AppState,
    pub mw: Middleware,
}

impl Server {
    pub fn new(state: AppState) -> Self {
        let mw = Middleware::from(&state.cfg);
        Self { state, mw }
    }

    pub fn router(&self) -> Router {
        // Build concrete middleware layers directly from config
        let request_id_header = self.state.cfg.request_id_header.clone();
        let request_id_stack = ServiceBuilder::new()
            .layer(SetRequestIdLayer::new(request_id_header.clone(), MakeRequestUuid))
            .layer(PropagateRequestIdLayer::new(request_id_header))
            .into_inner();

        let mut router = Router::new()
            .route("/health", get(health))
            .route("/todos", post(create_todo).get(get_all_todos))
            .with_state(self.state.clone())
            // innermost of these
            .layer(NormalizePathLayer::trim_trailing_slash())
            .layer(request_id_stack)
            .layer(TraceLayer::new_for_http()); // outermost of these

        if let Some(cors) = self.mw.cors_layer() {
            router = router.layer(cors);
        }

        router
    }
}
