use std::net::TcpListener;

use axum::{
    extract::Form, http::StatusCode, routing::get, routing::post, routing::IntoMakeService, Router,
    Server,
};
use hyper::server::conn::AddrIncoming;
use serde::Deserialize;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

#[derive(Debug, Deserialize)]
struct Subscription {
    name: String,
    email: String,
}

async fn subscribe(Form(_input): Form<Subscription>) -> StatusCode {
    StatusCode::OK
}

pub fn run(listener: TcpListener) -> hyper::Result<Server<AddrIncoming, IntoMakeService<Router>>> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe));

    let server = Server::from_tcp(listener)?.serve(app.into_make_service());

    Ok(server)
}
