# TASK.md — AI Agent Test Authoring Plan (Axum + SQLx + Shuttle)

## Objective
Create a comprehensive **automated test suite**—unit, component/middleware, and integration/E2E—that evaluates the robustness and quality of this Axum application. Use the repository’s `README.md` testing guidance as your blueprint and align test coverage with the app’s actual routes, middleware, and configuration.

## Inputs
- **Testing guide**: `README.md` (principles, patterns, DB strategies, org tips) :contentReference[oaicite:0]{index=0} :contentReference[oaicite:1]{index=1} :contentReference[oaicite:2]{index=2} :contentReference[oaicite:3]{index=3}
- **App surface**:
  - Routes: `/health`, `/todos` (GET all, POST create) :contentReference[oaicite:4]{index=4}; handlers for health/todos :contentReference[oaicite:5]{index=5} :contentReference[oaicite:6]{index=6} :contentReference[oaicite:7]{index=7}
  - Middleware stack: trace, normalize-path (trims trailing slash), request-id (set+propagate), optional CORS from config :contentReference[oaicite:8]{index=8} :contentReference[oaicite:9]{index=9}; behavior details & composition API :contentReference[oaicite:10]{index=10} :contentReference[oaicite:11]{index=11}
  - Config: request-id header name, timeout, CORS policy via env; loader rules :contentReference[oaicite:12]{index=12} :contentReference[oaicite:13]{index=13} :contentReference[oaicite:14]{index=14} :contentReference[oaicite:15]{index=15}
  - Startup: runs migrations; optional admin bearer header gate in `main` (runtime-only layer) :contentReference[oaicite:16]{index=16} :contentReference[oaicite:17]{index=17}
- **Dependencies to leverage**: axum, tower, tower-http, tokio, reqwest (tests), sqlx :contentReference[oaicite:18]{index=18} :contentReference[oaicite:19]{index=19} :contentReference[oaicite:20]{index=20}

---

## Scope & Milestones

### 1) Unit Tests (fast, in-memory)
Follow the README’s pattern: build a minimal `Router` and drive requests via `tower::ServiceExt::oneshot` (no sockets) :contentReference[oaicite:21]{index=21} :contentReference[oaicite:22]{index=22}.
- **Handlers**
  - `/health` returns `200` and expected body (“This is a health check”). Use a tiny router with just the route & state (if needed) :contentReference[oaicite:23]{index=23}.
  - `/todos`:
    - `GET /todos`: returns JSON array; on DB error maps to `500` with message path covered (assert status & error structure) :contentReference[oaicite:24]{index=24} :contentReference[oaicite:25]{index=25}.
    - `POST /todos`: validate behavior when SQL insert fails (currently uses `INSERT ...` with `fetch_all`—expect error path & `500`) :contentReference[oaicite:26]{index=26}.
- **State & services**
  - Add tests showing handlers consume `State` correctly (mirror README’s “hello uses state” style) :contentReference[oaicite:27]{index=27} :contentReference[oaicite:28]{index=28}.
  - If any repository traits exist, demonstrate mocking via trait substitution (see README mock example) :contentReference[oaicite:29]{index=29} :contentReference[oaicite:30]{index=30}.

**Deliverables (Unit)**
- `src/routes.rs` co-located `#[cfg(test)] mod tests` for pure handler logic, plus any domain logic tests (no I/O) :contentReference[oaicite:31]{index=31}.

### 2) Middleware/Composition Tests (component-level, no real sockets)
Construct `Server::router()` and assert:
- **Request ID propagation**—response includes configured header (default `x-request-id`) due to Set+Propagate stack :contentReference[oaicite:32]{index=32}; defaults in config :contentReference[oaicite:33]{index=33}.
- **Trailing slash normalization**—`/health/` works like `/health` (NormalizePathLayer) :contentReference[oaicite:34]{index=34}.
- **CORS policy**—when config is `Permissive` vs `Allow([...])` vs `Disabled`, assert presence/absence and shape of CORS headers :contentReference[oaicite:35]{index=35}.
- **Timeout layer** behavior (if wired in via `MiddlewareSuite::timeout`)—validate that very slow handlers receive a `timeout` response where applicable :contentReference[oaicite:36]{index=36}.

**Notes:** The admin bearer **ValidateRequestHeaderLayer** is applied in `main` (dependent on secrets). Treat this as a **runtime-only** concern; for tests, either (a) layer it explicitly on the test router for targeted negative tests, or (b) assert default behavior without it (no auth required) :contentReference[oaicite:37]{index=37}.

**Deliverables (Component)**
- `tests/middleware.rs` focusing on headers (request-id), path normalization, CORS variants.

### 3) Integration/E2E Tests (spawn server, real HTTP)
Use the README’s spawn-on-ephemeral-port pattern and drive with `reqwest` :contentReference[oaicite:38]{index=38} :contentReference[oaicite:39]{index=39} :contentReference[oaicite:40]{index=40}.
- **Happy paths**
  - `/health` is `200` via real HTTP.
  - `/todos` end-to-end: create (if DB reachable) then list; otherwise assert graceful error handling and status codes.
- **Negative paths**
  - Invalid payload on `POST /todos` → `400`/`422` (based on extractor behavior).
  - DB unavailable (point to non-existent DB) → app returns `5xx` on data routes, `/health` still `200`.
- **Cross-cutting**
  - Ensure request-id header is present end-to-end.
  - If you opt-in to an auth layer in test, assert `401/403` until correct `Authorization: Bearer` is supplied (simulated layer).

**Deliverables (Integration)**
- `tests/healthcheck.rs`, `tests/todos_flow.rs` (and any shared helpers in `tests/common/mod.rs`) :contentReference[oaicite:41]{index=41}.

### 4) Database Test Strategy
Implement **either** transaction-per-test **or** ephemeral-DB-per-suite as guided:
- **Transaction-per-test** (fast): begin transaction, exercise queries, rollback :contentReference[oaicite:42]{index=42} :contentReference[oaicite:43]{index=43}.
- **Ephemeral DB** (fidelity): create unique DB, run `sqlx::migrate!("./migrations")`, drop at teardown; prefer unique names to support parallelism :contentReference[oaicite:44]{index=44} :contentReference[oaicite:45]{index=45}.
- **Containers (optional)**: `testcontainers` Postgres for portability (use when local DB not available) :contentReference[oaicite:46]{index=46}.

**Execution env**
- Respect `TEST_DATABASE_URL` and ensure migrations run before tests that need a schema :contentReference[oaicite:47]{index=47}.
- App itself also performs migrations at startup; for spawned-app tests against a fresh DB, either rely on app startup migration or run migrations in test setup :contentReference[oaicite:48]{index=48}.

---

## Test Matrix (minimum)
1. **Health**
   - GET `/health` → 200 body text (no JSON) :contentReference[oaicite:49]{index=49}.
   - GET `/health/` → 200 (normalize-path) :contentReference[oaicite:50]{index=50}.
2. **Request ID**
   - Any route includes `x-request-id` (or overridden name) in response when router stack is used :contentReference[oaicite:51]{index=51} :contentReference[oaicite:52]{index=52}.
3. **CORS**
   - `Permissive`: default allow headers/methods, wildcard-ish origins via helper; `Disabled`: none; `Allow([...])`: exact list :contentReference[oaicite:53]{index=53}.
4. **Todos**
   - `GET /todos`: 200 JSON array when DB OK; 500 on DB error path with message (assert format) :contentReference[oaicite:54]{index=54}.
   - `POST /todos`: malformed body → 400/422; DB insert error path → 500 (current query style) :contentReference[oaicite:55]{index=55}.
5. **Timeout (optional)**
   - If a slow test handler is routed with `MiddlewareSuite::timeout`, assert timeout response :contentReference[oaicite:56]{index=56}.

---

## Implementation Notes

- **Patterns to copy from README**  
  - Unit HTTP testing via oneshot (no sockets) :contentReference[oaicite:57]{index=57}.  
  - State-backed handler test example (clone & adapt) :contentReference[oaicite:58]{index=58} :contentReference[oaicite:59]{index=59}.  
  - Spawning an app for E2E and driving with `reqwest` :contentReference[oaicite:60]{index=60} :contentReference[oaicite:61]{index=61}.
- **Organization**  
  - Unit tests live next to code; integration tests under `tests/` with descriptive filenames; share helpers under `tests/common` :contentReference[oaicite:62]{index=62}.
- **Running**  
  - `cargo test` (optionally with logs & SQLx offline, and `TEST_DATABASE_URL` set) :contentReference[oaicite:63]{index=63}.
- **Flakiness & Perf**  
  - Use `tokio::time::timeout`, deterministic seeds, parallelize sensibly; serialize DB-dependent tests per transaction if needed :contentReference[oaicite:64]{index=64}.
- **Library versions** (align helpers with available crates in `Cargo.toml`) :contentReference[oaicite:65]{index=65} :contentReference[oaicite:66]{index=66}.

---

## Output Structure (what you must produce)
- **Unit tests**
  - `src/routes.rs` → `#[cfg(test)] mod tests` covering health and todos handlers (happy + error paths).
  - Additional `#[cfg(test)]` blocks where domain logic exists.
- **Component/middleware tests**
  - `tests/middleware.rs` (request-id, normalize-path, CORS, timeout).
- **Integration/E2E**
  - `tests/healthcheck.rs`
  - `tests/todos_flow.rs`
  - `tests/common/mod.rs` (spawn helpers, DB setup utils, fixture builders).
- **DB helpers**
  - `tests/common/db.rs` for **transaction-per-test** and/or **ephemeral DB** creation + `sqlx::migrate!("./migrations")` runner (when needed) :contentReference[oaicite:67]{index=67}.

---

## Acceptance Criteria (Definition of Done)
- Tests compile and pass via `cargo test` (with `TEST_DATABASE_URL` configured when DB tests run) :contentReference[oaicite:68]{index=68}.
- Coverage includes:
  - All public routes (health, todos GET/POST) with positive & negative cases.
  - Middleware behaviors: request-id header, path normalization, CORS modes.
  - DB strategy implemented (transaction or ephemeral) with rollback/teardown.
- README guidelines are clearly followed (patterns, organization, run commands, checklist) :contentReference[oaicite:69]{index=69} :contentReference[oaicite:70]{index=70}.
- A short `TESTING_NOTES.md` (optional) explains which DB strategy you chose and how to run only DB tests locally.

---

## Contingencies & Tips
- If a Postgres server isn’t available, fall back to `testcontainers` (guarded by a feature flag or env switch) :contentReference[oaicite:71]{index=71}.
- If the admin bearer layer is required for certain deployments, simulate it by layering `ValidateRequestHeaderLayer::bearer("test")` onto the test router and assert `401/403` vs success with header present :contentReference[oaicite:72]{index=72}.
- For flaky network-bound tests, tune client timeouts and use `tokio::time::timeout` around awaits (document non-default timeouts) :contentReference[oaicite:73]{index=73}.

---

## Kickstart Snippets

### Minimal oneshot test template (clone per handler)
```rust
use axum::{Router, routing::get, http::{Request, StatusCode}};
use tower::ServiceExt;
use hyper::Body;

#[tokio::test]
async fn health_is_ok() {
    let app = Router::new().route("/health", get(crate::routes::health));
    let res = app.oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}
```
(Technique mirrors README’s unit approach) :contentReference[oaicite:74]{index=74}

### Spawned app test (HTTP)
```rust
#[tokio::test]
async fn e2e_healthcheck() {
    let (base_url, _server) = tests::common::spawn_app().await;
    let res = reqwest::Client::new()
        .get(format!("{base_url}/health"))
        .send().await.unwrap();
    assert!(res.status().is_success());
}
```
(Matches README’s E2E style) :contentReference[oaicite:75]{index=75}

---

**Proceed to implement the above. Aim for small, focused tests with clear assertions, fast feedback, and deterministic setups.**

