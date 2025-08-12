-- migrations/0001_create_todos.sql
CREATE TABLE IF NOT EXISTS todos (
  id   BIGSERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT NOT NULL,
  done  BOOLEAN NOT NULL DEFAULT FALSE
);
