# Database

This service uses Postgres with SQLx. The database connection is provisioned by Shuttle and injected as a PgPool. Migrations are managed with SQLx’s embedded migrations and are executed automatically on startup.

- Driver/ORM: SQLx (async, compile-time checked queries when enabled)
- Pool type: sqlx::PgPool
- Provisioning: Shuttle managed Postgres via attribute injection
- Migrations: SQL files in migrations/ embedded by sqlx::migrate!


## Connection provisioning

The PgPool is provided by Shuttle at startup using the Postgres resource attribute. It is then stored in AppState and used by handlers.

Code references:

- src/main.rs (entrypoint)

```rust
#[shuttle_runtime::main]
async fn axum(
    #[shuttle_shared_db::Postgres] pool: PgPool,
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleAxum {
    // Run database migrations at startup
    sqlx::migrate!().run(&pool).await.expect("Failed to run Migrations :(");

    let cfg = ServerConfig::load_from_env().expect("config");
    let state = AppState::new(pool, cfg);
    let server = Server::new(state);
    // ...
}
```

- src/config/app_state.rs

```rust
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cfg: ServerConfig,
    pub started_at: Instant,
}
```

Handlers use the pool from AppState for queries, for example:

```rust
let todos = sqlx::query_as::<_, Todo>("SELECT * FROM todos")
    .fetch_all(&state.pool)
    .await?;
```

Notes:
- When running on Shuttle (local or deployed), the database is created and configured automatically for this service. You do not need to set DATABASE_URL manually in this template.
- If you choose to run outside Shuttle, you would create your own PgPool from a DATABASE_URL and still call sqlx::migrate!().run(&pool) on startup.


## Schema

The template ships with a single example table: todos.

Migration file: migrations/0001_create_todos.sql

```sql
-- migrations/0001_create_todos.sql
CREATE TABLE IF NOT EXISTS todos (
  id   BIGSERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  done  BOOLEAN NOT NULL DEFAULT FALSE
);
```

Model mapping (src/models/todo.rs):

```rust
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub done: bool,
}
```

Caveat (IDs): The current create path in routes creates a Todo with an ID derived from a UUID string parsed into i64, which will panic at runtime. Prefer letting the database assign BIGSERIAL IDs and returning the inserted row. Adjust insert queries accordingly as you evolve the template.


## Migrations

- Location: migrations/ at the project root
- Mechanism: sqlx::migrate! macro embeds SQL files at compile-time and applies them at runtime with .run(&pool)
- Ordering: files are applied in lexical order; use numeric prefixes like 0001_, 0002_, etc.

At startup, main.rs runs:

```rust
sqlx::migrate!().run(&pool).await?;
```

This ensures the database is up-to-date before serving requests.

### Adding a new migration

You have two options:

1) Manually create a file
- Create a new SQL file under migrations/ with the next number, e.g., migrations/0002_add_due_date.sql
- Put your DDL/DML statements in that file
- Rebuild and run the service; the new migration will be applied automatically

2) Use sqlx-cli (optional tooling)
- Install: cargo install sqlx-cli --locked --features native-tls,postgres
- Ensure you have a Postgres instance and DATABASE_URL set when using the CLI directly (not required when running via Shuttle)
- Create a migration (reversible):
  - sqlx migrate add -r "add_due_date"
  - This creates up/down SQL files under migrations/; edit them with your changes
- Apply migrations (outside Shuttle):
  - sqlx migrate run

### Conventions and best practices
- Always use numeric prefixes to guarantee order (0001_, 0002_, ...)
- Keep migrations immutable once merged; create a new numbered migration to change schema
- Prefer explicit SQL over ORMs’ implicit migrations for reviewability
- Test locally by starting the app and verifying startup applies migrations cleanly


## Query examples

- List todos

```rust
sqlx::query_as::<_, Todo>("SELECT * FROM todos")
    .fetch_all(&state.pool)
    .await
```

- Insert a todo (recommended approach letting DB assign id)

```rust
let todo = sqlx::query_as::<_, Todo>(
    r#"
    INSERT INTO todos (title, description, done)
    VALUES ($1, $2, $3)
    RETURNING id, title, description, done
    "#,
)
.bind(&create.title)
.bind(&create.description)
.bind(&create.done)
.fetch_one(&state.pool)
.await?;
```


## Troubleshooting

- Migration errors at startup
  - Ensure your SQL files are valid and ordered correctly
  - Verify the migrations/ directory exists and is committed
- Missing database locally
  - Use `cargo shuttle run` to start a managed Postgres instance for this service
- Query compile errors (with SQLx offline checks enabled)
  - Make sure the DATABASE_URL is set when using sqlx-cli features; Shuttle injection is separate from CLI usage

