// src/models/server.rs (composition point)
use crate::{
    config::AppState,
    middleware::{Middleware, MiddlewareSuite},
    routes::{health, create_todo, get_all_todos},
};
use axum::{
    Router,
    routing::{get, post}
};
use tower::ServiceBuilder;

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
        // Build base router with routes and state
        let mut router = Router::new()
            .route("/health", get(health))
            .route("/todos", post(create_todo).get(get_all_todos))
            .with_state(self.state.clone());

        // Apply layers in the right order. Remember: the *last* .layer() is the
        // outermost wrapper for requests.
        //
        // For request IDs + tracing the recommended flow is:
        //   set_x_request_id  ->  trace  ->  propagate_x_request_id
        //
        // normalize_path / timeout can be placed around that as you prefer.
        router = router
            .layer(self.mw.normalize_path())  // innermost of these
            .layer(self.mw.timeout())
            .layer(self.mw.request_id_stack())
            .layer(self.mw.trace());          // outermost of these

        // Add CORS if configured
        if let Some(cors) = self.mw.cors_layer() {
            router = router.layer(cors);
        }

        router
    }
}
