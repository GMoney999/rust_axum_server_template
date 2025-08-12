// src/middleware/middleware_suite.rs
use tower::Layer;
use tower_http::cors::CorsLayer;

#[allow(dead_code)]
pub trait MiddlewareSuite {
    fn trace<S>(&self) -> impl Layer<S> + Clone;
    fn normalize_path<S>(&self) -> impl Layer<S> + Clone;
    fn request_id_stack<S>(&self) -> impl Layer<S> + Clone;
    fn timeout<S>(&self) -> impl Layer<S> + Clone;
    fn cors_layer(&self) -> Option<CorsLayer>;
}
