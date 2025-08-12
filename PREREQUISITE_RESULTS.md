# PREREQUISITE RESULTS

Date: 2025-08-12

1) Fix ID generation per docs/database/README.md caveat
- Problem: The create path generated a Todo::from(CreateTodo) with an id from Uuid::new_v4().to_string().parse::<i64>().unwrap(), which can panic and also conflicts with BIGSERIAL.
- Changes made:
  - Removed the Todo::from constructor and any UUID->i64 generation from src/models/todo.rs.
  - Updated src/routes/routes.rs create_todo to let Postgres assign the BIGSERIAL id and RETURNING the inserted row fields.
  - New insert query:
    INSERT INTO todos (title, description, done)
    VALUES ($1, $2, $3)
    RETURNING id, title, description, done
  - Handler now returns (201 Created, JSON of the inserted Todo).
- Rationale: Aligns runtime behavior with schema (id BIGSERIAL) and avoids parse panic.

2) Deployment CI consideration per docs/deployment/README.md
- Current state:
  - .github/workflows/ci.yml already runs format, clippy, and cargo test.
  - .github/workflows/shuttle-deploy.yml exists and handles deployment with an API key.
- Recommendation:
  - Ensure the deploy workflow either depends on tests or includes a separate test job. Since ci.yml already runs tests, a common pattern is to set the deploy workflow to trigger only on main and/or require branch protection rules so deploy happens only after CI passes. Alternatively, add a test job in shuttle-deploy.yml before the deploy step.

Notes
- No schema changes were required; migrations already define id BIGSERIAL.
- No API contract changes other than returning the created Todo JSON (status 201) instead of a string message.

