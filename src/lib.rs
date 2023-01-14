use axum::{http::StatusCode, routing::get, Router};
use std::net::SocketAddr;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[tokio::main]
pub async fn run() {
    let app = Router::new().route("/health_check", get(health_check));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
