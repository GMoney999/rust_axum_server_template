#[allow(clippy::module_inception)]
mod routes;
pub use routes::{create_todo, get_all_todos, health};
