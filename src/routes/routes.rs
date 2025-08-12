use crate::{
    config::AppState,
    models::{Todo, CreateTodo}
};
use axum::{
    extract::{State, Json},
    response::IntoResponse,
    http::StatusCode,
    Json as JsonData,
};

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "This is a health check")
}

pub async fn get_all_todos(
    State(state): State<AppState>
) -> Result<impl IntoResponse, impl IntoResponse> {
    let todos = match sqlx::query_as::<_, Todo>(
        "SELECT * FROM todos"
    )
        .fetch_all(&state.pool).await {
        Ok(todos) => todos,
        Err(e) => { return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to fetch all To-Dos: {}", e)))}
    };

    Ok(Json(todos))
}

pub async fn create_todo(
    State(state): State<AppState>,
    JsonData(json): Json<CreateTodo>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let todo = Todo::from(json);

    if let Err(e) = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (id, title, description, done) VALUES ($1, $2, $3, $4)"
    )
        .bind(&todo.id)
        .bind(&todo.title)
        .bind(&todo.description)
        .bind(&todo.done)
        .fetch_all(&state.pool).await {
            return Err((StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create Todo: {}", e)));
    }

    Ok(format!("Successfully created user {}", todo.id))
}
