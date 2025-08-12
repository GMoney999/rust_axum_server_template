use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub done: bool,
}

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
    pub fn new(id: i64, title: String, description: String, done: bool) -> Self {
        Self { id, title, description, done }
    }

    pub fn from(create_todo: CreateTodo) -> Self {
        let id = Uuid::new_v4().to_string().parse::<i64>().unwrap();
        Self {
            id,
            title: create_todo.title,
            description: create_todo.description,
            done: create_todo.done
        }
    }
}
