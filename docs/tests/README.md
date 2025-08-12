# Tests and how to run them

This project now includes a basic test scaffold per TASK.md:

- Unit tests:
  - src/routes/routes.rs (health and POST /todos malformed-body path)
- Component/middleware tests:
  - tests/middleware.rs (request-id header presence)
- Integration/E2E:
  - tests/healthcheck.rs (placeholder)
  - tests/todos_flow.rs (placeholder)
- Shared helpers:
  - tests/common/{mod.rs, db.rs}

Running tests:
- Basic: `cargo test`
- With DB-dependent tests (when implemented), set TEST_DATABASE_URL to a Postgres server, e.g.:
  - TEST_DATABASE_URL=postgres://postgres:postgres@127.0.0.1:5432/postgres cargo test

Notes:
- The create route now returns 201 with the inserted Todo JSON, and IDs are assigned by the database (BIGSERIAL).
- Request ID header defaults to `x-request-id`.

