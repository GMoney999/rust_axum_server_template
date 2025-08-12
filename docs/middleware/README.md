# Middleware

This service assembles its HTTP middleware stack explicitly in code, with small, composable layers. This document explains each layer, the precise order in which they are applied, and how CORS is conditionally included based on configuration.

Source of truth:
- Composition: src/models/server.rs (router construction and layer ordering)
- Layers: src/middleware/middleware.rs and src/middleware/middleware_suite.rs
- Config: src/config/server_config.rs

## Layer catalog

- TraceLayer (tower-http)
  - Purpose: Emits structured logs/spans for requests and responses, including latency and status codes.
  - Behavior: Wraps every request/response so that timing and errors can be recorded.

- Request ID stack (tower-http)
  - SetRequestIdLayer + PropagateRequestIdLayer
  - Purpose: Generate a UUID for each incoming request if one is not already present, and propagate it downstream and in responses using a configurable header (default: x-request-id).
  - Benefit: Enables correlation of logs and traces across layers and services.

- NormalizePathLayer (tower-http)
  - Purpose: Trims trailing slashes to normalize route matching (e.g., /todos/ -> /todos).
  - Benefit: Reduces 404s from minor client path variations and simplifies routing.

- CORS (tower-http, optional)
  - Purpose: Adds CORS headers when enabled. Supports permissive mode or an explicit allow-list of origins.
  - Inclusion: Conditionally layered on top of the stack depending on ServerConfig.cors.

## Order of application

Within Server::router() (src/models/server.rs), the stack is assembled as follows:

Innermost -> Outermost (base stack inside the Router):
1) NormalizePathLayer::trim_trailing_slash()
2) Request ID stack: SetRequestIdLayer + PropagateRequestIdLayer
3) TraceLayer::new_for_http()

If CORS is enabled, it is applied OUTSIDE the base stack (i.e., even more outer):
4) CORS layer (optional)

Finally, at the very outer edge, main.rs may apply an additional ValidateRequestHeaderLayer::bearer if an ADMIN_TOKEN is configured. That outer bearer layer is not part of this module, but it will sit above the entire stack described here.

Why this order?
- NormalizePath runs early so that all subsequent layers and handlers see canonicalized paths.
- Request ID is inside Trace so that the request id is already set when logging/trace events occur during handling; the Trace layer still wraps the entire lifecycle to record timing and status.
- CORS sits outermost for simple cross-origin preflight handling and to guarantee responses include the appropriate headers regardless of inner behavior.

## Conditional CORS inclusion

CORS behavior is driven by ServerConfig.cors (src/config/server_config.rs), which is loaded from environment variables:
- CORS_DISABLED: if set to any non-empty value, CORS is Disabled (no CORS headers are added).
- CORS_ALLOWED_ORIGINS: comma-separated list of allowed origins; when provided (and not disabled), CORS is Allow([...]).
- Otherwise, the default is Permissive, which allows common methods/headers and any origin.

Precedence: Disabled > Allow-list > Permissive(default)

Implementation details:
- Middleware::cors_layer() returns:
  - None when Disabled (no layer is added to the router)
  - Some(CorsLayer) when Allow-list or Permissive
- Defaults applied by with_defaults():
  - Allowed methods: GET, POST, PUT, PATCH, DELETE
  - Allowed headers: content-type, authorization
  - Credentials: allowed (allow_credentials(true))
  - Origins: Any (Permissive) or an explicit list (Allow([...]))

## Configuration quick reference

- REQUEST_ID_HEADER
  - Default: x-request-id
  - Used by Request ID stack to set/propagate a UUID per request.

- TIMEOUT_SECS
  - Default: 15
  - Available in ServerConfig and a TimeoutLayer helper exists, but the timeout layer is not currently attached in Server::router(). You can add it if desired.

- CORS_DISABLED
  - Any non-empty value disables CORS entirely.

- CORS_ALLOWED_ORIGINS
  - Comma-separated list of origins (e.g., https://app.example.com, https://admin.example.com)
  - Enables allow-list mode when present and not empty.

## Extending the stack

If you need to add or reorder layers, do so in src/models/server.rs so the composition point remains the single source of truth. Keep the following guidelines in mind:
- Put path normalization before routing-sensitive layers and handlers.
- Ensure Request ID is available as early as possible to correlate logs.
- Keep tracing as an outer wrapper to capture total latency and status.
- Place CORS near the outer edge so preflights and responses are consistently decorated.
- If adding auth layers, place them outermost to short-circuit unauthorized requests early.
