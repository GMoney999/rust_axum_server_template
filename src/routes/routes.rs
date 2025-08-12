use crate::{
    config::AppState,
    models::{CreateTodo, Todo},
};
use axum::{
    Json as JsonData,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
};

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "This is a health check")
}

pub async fn get_all_todos(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let todos = match sqlx::query_as::<_, Todo>("SELECT * FROM todos")
        .fetch_all(&state.pool)
        .await
    {
        Ok(todos) => todos,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch all To-Dos: {}", e),
            ));
        }
    };

    Ok(Json(todos))
}

pub async fn create_todo(
    State(state): State<AppState>,
    JsonData(json): Json<CreateTodo>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // Let the database assign BIGSERIAL id and return the inserted row
    let inserted = match sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (title, description, done)
        VALUES ($1, $2, $3)
        RETURNING id, title, description, done
        "#,
    )
    .bind(&json.title)
    .bind(&json.description)
    .bind(json.done)
    .fetch_one(&state.pool)
    .await
    {
        Ok(todo) => todo,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to create Todo: {}", e),
            ));
        }
    };

    Ok((StatusCode::CREATED, Json(inserted)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppState, ServerConfig};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::{
        Router,
        routing::{get, post},
    };
    use sqlx::PgPool;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_unit_ok() {
        let app = Router::new().route("/health", get(health));
        let res = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn create_todo_malformed_body_returns_400() {
        // Provide dummy state so the extractor can resolve, but JSON extraction fails first
        let cfg = ServerConfig::default();
        let pool = PgPool::connect_lazy("postgres://127.0.0.1/postgres").expect("lazy pool");
        let state = AppState::new(pool, cfg);
        let app = Router::new()
            .route("/todos", post(create_todo))
            .with_state(state);

        let res = app
            .oneshot(
                Request::builder()
                    .uri("/todos")
                    .method("POST")
                    .body(Body::from("not-json"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(matches!(
            res.status(),
            StatusCode::BAD_REQUEST
                | StatusCode::UNPROCESSABLE_ENTITY
                | StatusCode::UNSUPPORTED_MEDIA_TYPE
        ));
    }
}
