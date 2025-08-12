# Deployment

This service is deployed with Shuttle. It uses Shuttle’s runtime entrypoint and managed resources for Postgres and Secrets.

- Runtime: shuttle-runtime with setup-otel-exporter feature
- Web adapter: shuttle-axum (Axum Router integration)
- Managed DB: shuttle-shared-db with Postgres and SQLx
- Secrets: injected via Shuttle Secret Store (ADMIN_TOKEN optional)
- Migrations: SQLx migrations run automatically at startup


## Shuttle runtime features used
- shuttle-runtime (features: setup-otel-exporter)
  - Enables OpenTelemetry export setup via Shuttle when configured
- shuttle-axum
  - Returns Axum Router through ShuttleAxum
- shuttle-shared-db (features: postgres, sqlx)
  - #[shuttle_shared_db::Postgres] injects a managed PgPool
- Secrets
  - #[shuttle_runtime::Secrets] injects SecretStore for runtime secrets

Relevant files:
- Cargo.toml: shuttle-runtime, shuttle-axum, shuttle-shared-db dependencies and features
- Shuttle.toml: project name for deployment
- migrations/: SQLx migrations applied on startup
- src/main.rs: Shuttle entrypoint, DB pool and secrets injection, auth layer, migrations


## Local development (cargo shuttle run)
Prerequisites:
- Rust toolchain installed
- Shuttle CLI: cargo install cargo-shuttle
- Optional: psql or a DB GUI if you want to inspect the local database

Run locally:
- cargo shuttle run
  - Spins up the service locally using Shuttle’s adapter
  - Provisions a local Postgres instance for the #[shuttle_shared_db::Postgres] resource
  - Sets DATABASE_URL internally; the app runs sqlx::migrate!() at startup

Manage secrets (local):
- shuttle secret set ADMIN_TOKEN
  - The app will guard all routes behind Authorization: Bearer <ADMIN_TOKEN> if set

Useful environment variables (read by the app):
- REQUEST_ID_HEADER (default: x-request-id)
- TIMEOUT_SECS (default: 15)
- CORS_DISABLED (any value disables CORS layer)
- CORS_ALLOWED_ORIGINS (comma-separated allow-list)

Inspect logs:
- Use RUST_LOG to control verbosity, for example:
  - RUST_LOG=info,tower_http=info,sqlx=warn cargo shuttle run

Database:
- Migrations are applied automatically on boot
- The injected PgPool is used by handlers via AppState


## Production deployment (Shuttle)
One-time setup:
- Install Shuttle CLI: cargo install cargo-shuttle
- Login: cargo shuttle login
- Ensure Shuttle.toml name is unique (name = "axum-server-shuttle" by default)

Set production secrets:
- cargo shuttle secret set ADMIN_TOKEN
  - Repeat per secret you need

Deploy:
- cargo shuttle deploy
  - Builds and deploys the service to Shuttle

Status and logs:
- cargo shuttle status
- cargo shuttle logs -f

Rollouts:
- Re-run cargo shuttle deploy to ship a new version

Database:
- The Postgres database is managed by Shuttle for this service
- Migrations run automatically at startup; ensure migrations/ is committed


## GitHub Actions CI/CD reference
There is no workflow checked into this repository yet. The following example shows a minimal deploy workflow you can add at .github/workflows/shuttle-deploy.yml.

Name: Shuttle Deploy
on:
  push:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Install cargo-shuttle
        run: cargo install cargo-shuttle --locked

      - name: Deploy to Shuttle
        env:
          SHUTTLE_API_KEY: ${{ secrets.SHUTTLE_API_KEY }}
        run: |
          # Authenticate non-interactively using the API key secret
          shuttle login --api-key "$SHUTTLE_API_KEY"
          cargo shuttle deploy --no-confirm

Notes:
- Create a repository secret named SHUTTLE_API_KEY with your Shuttle API key
- cargo shuttle deploy requires the project name from Shuttle.toml
- Ensure migrations/ is committed so production runs them on startup
- Consider adding a separate job for tests (cargo test) before deploy


## Troubleshooting
- Deploy fails due to missing secret: set it with cargo shuttle secret set <NAME>
- Migration errors: verify SQL files under migrations/ and that sqlx macros compile
- Auth blocked locally: unset or remove ADMIN_TOKEN secret for open endpoints during dev
- Logs missing: set RUST_LOG to increase verbosity

