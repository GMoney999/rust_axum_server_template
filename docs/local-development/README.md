# Local Development

This guide covers prerequisites, common commands, log viewing, and where to go for troubleshooting while working on this Axum + Shuttle REST API locally.


## Prerequisites
- Rust toolchain (rustup + cargo)
  - Install: https://rustup.rs
- Cargo Shuttle CLI
  - Install: cargo install cargo-shuttle --locked
- Optional tools
  - psql or a DB GUI (to inspect the local Postgres that Shuttle provisions)
  - curl or HTTP client (e.g., HTTPie, Postman) for testing endpoints

Environment variables you may use in dev:
- REQUEST_ID_HEADER (default: x-request-id)
- TIMEOUT_SECS (default: 15)
- CORS_DISABLED (any value disables CORS)
- CORS_ALLOWED_ORIGINS (comma-separated allow-list)
- RUST_LOG (tracing filter, e.g., info,tower_http=info,sqlx=warn)
- ADMIN_TOKEN (optional: enables Bearer auth on all routes when set)


## First run
Two common ways to run locally:

- cargo shuttle run
  - Uses Shuttle’s local runner and provisions a local Postgres instance automatically.
  - Applies SQLx migrations on startup.

- cargo run
  - Runs the binary directly with your system environment.
  - If you choose this route and the app requires a database, set DATABASE_URL accordingly and ensure Postgres is running.

Example with logging verbosity:
- RUST_LOG=info,tower_http=info,sqlx=warn cargo shuttle run


## Common commands
- Build: cargo build
- Run (Shuttle-managed dev): cargo shuttle run
- Run (direct): cargo run
- Test: cargo test
- Format: cargo fmt --all
- Lint: cargo clippy --all-targets -- -D warnings
- Manage a local dev ADMIN_TOKEN (Shuttle runner):
  - shuttle secrets add ADMIN_TOKEN
  - shuttle secrets list
  - shuttle secrets delete ADMIN_TOKEN

Useful endpoints to try once the server is running:
- GET /health
- GET /todos
- POST /todos with JSON: { "title": "t", "description": "d", "done": false }


## Viewing logs locally
- Control verbosity with RUST_LOG (defaults to info if unset in many setups)
  - Example: RUST_LOG=debug cargo shuttle run
- Request/response tracing is enabled via tower-http and tracing-subscriber
- Look for request IDs in logs (header: REQUEST_ID_HEADER, default x-request-id)

If you set ADMIN_TOKEN, all requests must include:
- Authorization: Bearer {{ADMIN_TOKEN}}

Example curl:
- curl -H "Authorization: Bearer {{ADMIN_TOKEN}}" http://localhost:8000/health

Replace {{ADMIN_TOKEN}} with your environment’s token value without echoing secrets to your shell history.


## Database and migrations
- Migrations live under migrations/
- When using cargo shuttle run, migrations are applied automatically at startup.
- For direct cargo run, ensure DATABASE_URL is set and reachable if you use the database.


## Troubleshooting and further docs
If something doesn’t work as expected, check these docs:
- Environment and Secrets: ../environment/README.md
- Deployment (incl. Shuttle commands): ../deployment/README.md
- Observability (logs/tracing): ../observability/README.md
- Database (SQLx/migrations): ../database/README.md
- Routes overview: ../routes/README.md
- Testing: ../testing/README.md

Common issues:
- 401 Unauthorized locally
  - You likely set ADMIN_TOKEN; include Authorization: Bearer {{ADMIN_TOKEN}} in requests or unset the token.
- CORS behavior unexpected
  - Verify CORS_DISABLED or CORS_ALLOWED_ORIGINS values.
- Migrations or DB errors
  - Ensure you’re using cargo shuttle run (provisions Postgres) or that DATABASE_URL is set when using cargo run.
