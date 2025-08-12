use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub done: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct UpdatedTodo {
    pub id: i64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub done: Option<bool>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub done: bool,
}

impl Todo {
    #[allow(dead_code)]
    pub fn new(id: i64, title: String, description: String, done: bool) -> Self {
        Self {
            id,
            title,
            description,
            done,
        }
    }
}
