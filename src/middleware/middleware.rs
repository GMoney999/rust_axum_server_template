// src/middleware/middleware.rs
use crate::config::{ServerConfig, CorsPolicy};
use crate::middleware::{MiddlewareSuite};

use std::time::Duration;
use axum::http::{header, HeaderName, HeaderValue, Method};
use tower::{Layer, ServiceBuilder};
use tower::timeout::TimeoutLayer;
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    normalize_path::NormalizePathLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};

#[derive(Clone, Debug)]
pub struct Middleware {
    request_id_header: HeaderName,
    timeout: Duration,
    cors: CorsPolicy,
}

impl From<&ServerConfig> for Middleware {
    fn from(cfg: &ServerConfig) -> Self {
        Self {
            request_id_header: cfg.request_id_header.clone(),
            timeout: cfg.timeout,
            cors: cfg.cors.clone(),
        }
    }
}

impl MiddlewareSuite for Middleware {
    fn trace<S>(&self) -> impl Layer<S> + Clone {
        TraceLayer::new_for_http()
            .on_request(DefaultOnRequest::new())
            .on_response(DefaultOnResponse::new())
            .on_failure(DefaultOnFailure::new())
    }
    fn normalize_path<S>(&self) -> impl Layer<S> + Clone {
        NormalizePathLayer::trim_trailing_slash()
    }
    fn request_id_stack<S>(&self) -> impl Layer<S> + Clone {
        let h = self.request_id_header.clone();
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::new(h.clone(), MakeRequestUuid))
            .layer(PropagateRequestIdLayer::new(h))
            .into_inner()
    }
    fn timeout<S>(&self) -> impl Layer<S> + Clone {
        TimeoutLayer::new(self.timeout)
    }
    fn cors_layer(&self) -> Option<CorsLayer> {
        match &self.cors {
            CorsPolicy::Disabled => None,
            CorsPolicy::Permissive => Some(CorsLayer::new().with_defaults().allow_origin(Any)),
            CorsPolicy::Allow(list) => {
                let allow_values: Vec<HeaderValue> = list.clone();
                Some(CorsLayer::new().with_defaults().allow_origin(AllowOrigin::list(allow_values)))
            }
        }
    }
}


trait CorsDefaultsExt { fn with_defaults(self) -> Self; }
impl CorsDefaultsExt for CorsLayer {
    fn with_defaults(self) -> Self {
        self.allow_methods(default_methods())
            .allow_headers(default_headers())
            .allow_credentials(true)
    }
}


#[inline] fn default_methods() -> [Method; 5] {
    [Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE]
}
#[inline] fn default_headers() -> [HeaderName; 2] {
    [header::CONTENT_TYPE, header::AUTHORIZATION]
}