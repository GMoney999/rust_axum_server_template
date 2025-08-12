pub mod db;

use std::{net::TcpListener};
use axum::Router;
use tokio::net::TcpListener as TokioListener;
use tokio::task::JoinHandle;
use hyper::Server;
use axum_server_shuttle::{models::Server, config::{AppState, ServerConfig}};
use sqlx::PgPool;

pub async fn spawn_app_with_pool(pool: PgPool) -> (String, JoinHandle<()>) {
    let cfg = ServerConfig::load_from_env().expect("cfg");
    let state = AppState::new(pool, cfg);
    let server = Server::new(state);
    let app = server.router();

    let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://{}:{}", addr.ip(), addr.port());

    let listener = TokioListener::from_std(listener).unwrap();
    let server = Server::from_tcp(listener.into_std().unwrap())
        .unwrap()
        .serve(app.into_make_service());

    let handle = tokio::spawn(async move {
        if let Err(e) = server.await { eprintln!("server error: {e}"); }
    });

    (url, handle)
}

