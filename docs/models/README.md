# Models

This document describes the data models used by the service for Todo resources, including their fields, serialization/deserialization behavior, and important caveats around ID handling.

## Overview

The Todo models live in `src/models/todo.rs` and consist of three structs:
- `CreateTodo` — payload for creating a new todo
- `UpdatedTodo` — payload for partially updating an existing todo
- `Todo` — the full todo model returned by the API and mapped from the database

## CreateTodo

Derived traits:
- `Deserialize`

Fields:
- `title: String` — required, the title of the todo
- `description: String` — required, a longer description
- `done: bool` — optional in input; defaults to `false`

Serde behavior:
- `done` has `#[serde(default)]`, which means if the client omits the field, it will default to `false` during deserialization.

Intended usage:
- Used as the request body when creating a new todo item.

## UpdatedTodo

Derived traits:
- `Deserialize`

Fields:
- `id: i64` — required, identifies which todo to update
- `title: Option<String>` — optional, new title if provided
- `description: Option<String>` — optional, new description if provided
- `done: Option<bool>` — optional, new done state if provided

Serde behavior:
- Optional fields (`Option<…>`) may be omitted by the client to leave them unchanged; when present, they indicate which fields should be updated.

Intended usage:
- Used as the request body for partial updates (PATCH-like semantics). Only supplied fields are considered for changes.

Note:
- This struct is currently marked with `#[allow(dead_code)]`, which may indicate it is not yet used in the request handling code.

## Todo

Derived traits:
- `Serialize`
- `sqlx::FromRow`

Fields:
- `id: i64` — unique identifier of the todo (commonly maps to a BIGINT in the database)
- `title: String`
- `description: String`
- `done: bool`

Serde behavior:
- `Todo` is serialized in responses. It does not implement `Deserialize` because it is not expected to be received from clients as-is.

Database mapping:
- `sqlx::FromRow` allows mapping database rows directly into this struct when querying with SQLx.

## ID handling caveats

- Type: The `id` across models is an `i64`, which typically corresponds to a BIGINT in SQL databases.
- Construction: The `Todo::from(create: CreateTodo)` helper currently generates an ID by creating a `Uuid` and attempting to parse it into an `i64`:
  ```rust
  let id = Uuid::new_v4().to_string().parse::<i64>().unwrap();
  ```
  This will panic at runtime because a UUID string (e.g., `"550e8400-e29b-41d4-a716-446655440000"`) cannot be parsed into an `i64`.
- Recommendation: Prefer one of the following approaches:
  - Let the database assign the ID (e.g., `SERIAL`/`BIGSERIAL`/auto-increment) and return it from the insert query.
  - If UUIDs are desired, change the `id` type to `Uuid` (or `String`) consistently across models and database schema.

Until this is addressed, avoid using `Todo::from(CreateTodo)` in code paths; instead, insert via the database and map back into `Todo` using the returned row.

