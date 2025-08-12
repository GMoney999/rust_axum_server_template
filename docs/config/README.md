# Configuration

This document describes how configuration is loaded, the environment variables it honors (with defaults), and the structure of the application state (AppState).

## Overview

Configuration is centered around the `ServerConfig` struct and is injected into handlers via `AppState`.

- Source of truth: `src/config/server_config.rs` and `src/config/app_state.rs`
- Loading strategy: values are read from environment variables when available; otherwise sensible defaults are used.

## Environment variables

The server recognizes the following environment variables at startup (read by `ServerConfig::load_from_env()`):

- REQUEST_ID_HEADER
  - Purpose: Header name used to track a unique request ID.
  - Type: HTTP header name (lowercase recommended).
  - Default: x-request-id
  - Example: REQUEST_ID_HEADER=x-correlation-id

- TIMEOUT_SECS
  - Purpose: Global per-request handler timeout in seconds.
  - Type: u64 (integer seconds)
  - Default: 15
  - Example: TIMEOUT_SECS=30

- CORS_DISABLED
  - Purpose: Disables CORS headers entirely when present.
  - Type: flag (any non-empty presence enables it)
  - Default: not set (CORS is permissive unless allow-list is provided)
  - Example: CORS_DISABLED=1

- CORS_ALLOWED_ORIGINS
  - Purpose: Comma-separated allow-list of origins.
  - Type: comma-separated list of strings (each must be a valid header value)
  - Default: not set
  - Example: CORS_ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com

CORS resolution precedence (highest to lowest):
1) CORS_DISABLED is set -> CORS is Disabled
2) CORS_ALLOWED_ORIGINS has at least one valid origin -> CORS is Allow([...])
3) Otherwise -> CORS is Permissive (allow everything)

Additional environment variables commonly used by the runtime/tooling:

- RUST_LOG
  - Purpose: Controls logging/trace filtering (via tracing_subscriber EnvFilter).
  - Default in code when RUST_LOG is unset: info,tower_http=info,sqlx=warn
  - Example: RUST_LOG=debug,tower_http=debug,sqlx=info

Secrets (in deployment) via Shuttle SecretStore:

- ADMIN_TOKEN
  - Purpose: If provided, all routes are protected by a static Bearer token.
  - Usage header: Authorization: Bearer <ADMIN_TOKEN>
  - Source: Shuttle secret store (not a plain env var)

Database:
- The project uses Shuttle's managed Postgres (`#[shuttle_shared_db::Postgres]`).
- Migrations located in `migrations/` are run automatically at startup.

## Defaults summary

- request_id_header: "x-request-id"
- timeout: 15 seconds
- cors: Permissive (unless overridden by CORS_DISABLED or CORS_ALLOWED_ORIGINS)

## AppState layout

Defined in `src/config/app_state.rs`:

- pool: sqlx::PgPool
  - Shared database connection pool.
- cfg: ServerConfig
  - The loaded server configuration (see below).
- started_at: std::time::Instant
  - Timestamp when the server started (currently not externally exposed; used for diagnostics or uptime calculations).

## ServerConfig layout

Defined in `src/config/server_config.rs`:

- request_id_header: axum::http::HeaderName
  - Header used to read/set a request ID (e.g., "x-request-id").
- timeout: std::time::Duration
  - Global handler timeout.
- cors: CorsPolicy
  - CORS behavior. One of:
    - Permissive: allow all
    - Allow(Vec<HeaderValue>): explicit allow-list
    - Disabled: no CORS headers

## How values are loaded

`ServerConfig::load_from_env()` applies the following logic:

- REQUEST_ID_HEADER: if set, parsed into HeaderName; falls back to "x-request-id".
- TIMEOUT_SECS: if set, parsed as u64 seconds; falls back to 15 seconds.
- CORS:
  - If CORS_DISABLED is set -> Disabled
  - Else if CORS_ALLOWED_ORIGINS is set and non-empty -> Allow(list)
  - Else -> Permissive

Errors during parsing (e.g., invalid REQUEST_ID_HEADER or an invalid origin string) will cause startup to fail with a helpful message.

## Example .env

The following examples show common setups for local development vs. stricter environments.

Example: local development (permissive CORS, shorter timeouts)

REQUEST_ID_HEADER=x-request-id
TIMEOUT_SECS=15
# CORS defaults to Permissive if neither of the following are set
# CORS_DISABLED=
# CORS_ALLOWED_ORIGINS=
# Logging
RUST_LOG=info,tower_http=info,sqlx=warn

Example: staging/production (allow-list CORS and longer timeout)

REQUEST_ID_HEADER=x-request-id
TIMEOUT_SECS=30
CORS_ALLOWED_ORIGINS=https://app.example.com,https://admin.example.com
# Lock down all routes behind a single token via Shuttle secrets (not .env):
# ADMIN_TOKEN is provided via deployment secrets store
RUST_LOG=info,tower_http=info,sqlx=warn

## Where this is used in code

- src/main.rs
  - Loads config: `let cfg = ServerConfig::load_from_env()?;`
  - Builds `AppState::new(pool, cfg)` and constructs the router.
  - Applies a Bearer token layer if `ADMIN_TOKEN` secret is provided.

- src/config/server_config.rs
  - Defines `ServerConfig` and `CorsPolicy` and implements `load_from_env` with the precedence rules above.

- src/config/app_state.rs
  - Defines the `AppState` struct used throughout the server.
