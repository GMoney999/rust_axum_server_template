# Testing Strategy for Axum Services

This document outlines a practical approach to testing an Axum-based service. It covers:

- Unit tests for handlers, extractors, and business logic
- Integration tests for end-to-end HTTP flows
- Database testing strategies, including ephemeral and transactional approaches
- Tips for test isolation, speed, and maintainability

The examples assume Rust, Axum, Tokio, and common libraries like tower, reqwest, and SQLx/SeaORM/Diesel (adapt as needed).


## General Principles

- Prefer pure functions for domain logic: unit test without frameworks.
- Keep handlers thin: delegate to services that can be mocked in unit tests.
- For HTTP-level behavior, use Axum's Router and tower::ServiceExt::oneshot in unit-like tests.
- For end-to-end verification, start the app on an ephemeral port and drive it with reqwest.
- Make tests deterministic and isolated: do not depend on global state; reset DB state per test.


## Unit Tests

Unit tests should run in-memory where possible and avoid real networking. For handlers, build a minimal Router with only the routes and state the test needs.

### Example: Handler unit test with Router and oneshot

```rust
use axum::{routing::get, Router, http::{Request, StatusCode}};
use tower::ServiceExt; // for `oneshot`
use hyper::Body;

async fn health() -> &'static str { "ok" }

#[tokio::test]
async fn health_returns_ok() {
    let app = Router::new().route("/health", get(health));

    let response = app
        .oneshot(Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body_bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body_bytes[..], b"ok");
}
```

### Example: Testing handlers that depend on state

Extract shared state with axum::extract::State and pass a test double in unit tests.

```rust
use std::sync::Arc;
use axum::{extract::State, routing::get, Router, Json, http::{Request, StatusCode}};
use serde::Serialize;
use tower::ServiceExt;
use hyper::Body;

#[derive(Clone, Default)]
struct AppState {
    greeting: String,
}

#[derive(Serialize)]
struct Greeting { message: String }

async fn hello(State(state): State<Arc<AppState>>) -> Json<Greeting> {
    Json(Greeting { message: state.greeting.clone() })
}

#[tokio::test]
async fn hello_uses_state() {
    let state = Arc::new(AppState { greeting: "hi".into() });
    let app = Router::new().route("/hello", get(hello)).with_state(state);

    let res = app
        .oneshot(Request::builder().uri("/hello").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
}
```

### Example: Mocking a repository/service

Abstract DB access behind a trait so you can substitute a mock in unit tests.

```rust
#[async_trait::async_trait]
pub trait UserRepo: Send + Sync {
    async fn get_name(&self, id: i64) -> anyhow::Result<Option<String>>;
}

pub struct Handler<R: UserRepo> { repo: R }

impl<R: UserRepo> Handler<R> {
    pub fn new(repo: R) -> Self { Self { repo } }
    pub async fn handle(&self, id: i64) -> String {
        self.repo.get_name(id).await.ok().flatten().unwrap_or_else(|| "unknown".into())
    }
}

struct MockRepo;

#[async_trait::async_trait]
impl UserRepo for MockRepo {
    async fn get_name(&self, _id: i64) -> anyhow::Result<Option<String>> {
        Ok(Some("alice".into()))
    }
}

#[tokio::test]
async fn handler_uses_repo() {
    let h = Handler::new(MockRepo);
    assert_eq!(h.handle(1).await, "alice");
}
```


## Integration Tests

Integration tests run the full stack: HTTP server, routing, middleware, and real DB (or a close substitute). Put these in tests/ so they compile as separate crates.

### Spawning the app on an ephemeral port

Have a helper that builds your Router and returns something that can be bound and spawned for tests.

```rust
use std::net::TcpListener;
use axum::Router;
use tokio::net::TcpListener as TokioListener;
use tokio::task::JoinHandle;
use hyper::Server;

pub fn build_app() -> Router {
    // construct your production router here
    Router::new()
}

pub async fn spawn_app() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}:{}", addr.ip(), addr.port());

    let listener = TokioListener::from_std(listener).unwrap();
    let app = build_app();

    let server = Server::from_tcp(listener.into_std().unwrap())
        .unwrap()
        .serve(app.into_make_service());

    let handle = tokio::spawn(async move {
        if let Err(e) = server.await { eprintln!("server error: {e}"); }
    });

    (url, handle)
}
```

### Driving requests with reqwest

```rust
#[tokio::test]
async fn healthcheck_is_ok() {
    let (base_url, _server) = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client.get(format!("{base_url}/health")).send().await.unwrap();
    assert!(res.status().is_success());
}
```


## Database Testing Strategies

Choose a strategy that balances isolation, speed, and fidelity.

- Transaction-per-test (fast): begin a transaction before each test; roll back at the end. Requires your data access layer to accept a transaction handle.
- Ephemeral database per test suite (fidelity): create a fresh DB/schema for tests, run migrations, then drop it after the suite.
- Containers (portability): use testcontainers to spawn Postgres/MySQL for tests; ensures parity with production.

### Example: SQLx with transaction-per-test

```rust
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn setup_pool() -> PgPool {
    // Use a dedicated test DB URL, not production.
    let database_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    PgPoolOptions::new().max_connections(5).connect(&database_url).await.unwrap()
}

#[tokio::test]
async fn creates_user() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    // call functions that accept `&mut tx`
    // create_user(&mut tx, ...).await.unwrap();

    tx.rollback().await.unwrap();
}
```

### Example: Creating an ephemeral DB/schema for the suite

```rust
use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
use uuid::Uuid;

pub async fn setup_ephemeral_db() -> (String, PgPool) {
    let base_url = std::env::var("TEST_DATABASE_URL").unwrap(); // points to the server, e.g. postgres://user:pass@localhost:5432/postgres
    let db_name = format!("test_{}", Uuid::new_v4());

    // connect to default DB
    let admin_pool = PgPoolOptions::new().max_connections(1).connect(&base_url).await.unwrap();
    admin_pool.execute(format!("CREATE DATABASE {db_name}").as_str()).await.unwrap();

    let db_url = format!("{}/{}", base_url.trim_end_matches(|c| c == '/' || c == ' '), db_name);
    let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await.unwrap();

    // run migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    (db_url, pool)
}
```

Drop the database in a test teardown or at process exit. For concurrent tests, prefer unique DB names per test.

### Example: Testcontainers for Postgres

```rust
use testcontainers::{clients::Cli, images::postgres::Postgres};

#[tokio::test]
async fn with_container() {
    let docker = Cli::default();
    let node = docker.run(Postgres::default());
    let port = node.get_host_port_ipv4(5432);
    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    // connect, migrate, run tests...
}
```


## Organizing Tests

- Place unit tests next to the modules they test using #[cfg(test)] mod tests.
- Place integration tests under tests/ with descriptive filenames, e.g., tests/healthcheck.rs, tests/users_flow.rs.
- Share helpers via a tests/common/mod.rs module.
- Avoid global mutable state; prefer per-test setup helpers.


## Axum-specific Tips

- Use tower::ServiceExt::oneshot for fast, no-socket HTTP tests.
- Test middleware by composing a minimal Router with only the middleware under test.
- If using axum_extra::extract::cookie or sessions, inject signed/unsigned key material from test fixtures.
- For JSON assertions, deserialize into structs or use serde_json::Value and assert on paths.
- For auth, provide helpers to mint test JWTs or API keys; never hardcode production secrets.


## Running Tests

- Run all tests: `cargo test`
- Run a single test file: `cargo test --test users_flow`
- Show logs: `RUST_LOG=debug cargo test -- --nocapture`
- With SQLx offline: `SQLX_OFFLINE=true cargo test` (if using SQLx offline features)

Ensure TEST_DATABASE_URL (or equivalent) is set for tests that need a database.


## Flakiness and Performance

- Use test-specific timeouts to catch hangs (e.g., tokio::time::timeout).
- Prefer deterministic seeds for randomized data (e.g., fake data generators with fixed RNG seeds).
- Parallelize where possible; for DB tests that share a database, serialize with a per-test transaction.


## Checklist

- [ ] Unit tests for core business logic (no I/O)
- [ ] Handler tests with Router and oneshot
- [ ] Integration tests with real HTTP and database
- [ ] Migrations applied for test DB
- [ ] Per-test isolation (transaction or ephemeral DB)
- [ ] Helpers for auth, state, and seeding
- [ ] CI: ensure services (DB) are available and TEST_DATABASE_URL is configured
