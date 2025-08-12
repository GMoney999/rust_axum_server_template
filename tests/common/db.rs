use sqlx::{PgPool, postgres::PgPoolOptions, Executor};
use uuid::Uuid;

pub async fn setup_ephemeral_db() -> (String, PgPool) {
    let base_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");
    let admin_pool = PgPoolOptions::new().max_connections(1).connect(&base_url).await.unwrap();
    let db_name = format!("test_{}", Uuid::new_v4());
    let create = format!("CREATE DATABASE \"{}\"", db_name);
    admin_pool.execute(create.as_str()).await.unwrap();

    let db_url = if base_url.ends_with('/') { format!("{base_url}{db_name}") } else { format!("{base_url}/{db_name}") };
    let pool = PgPoolOptions::new().max_connections(5).connect(&db_url).await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    (db_url, pool)
}

