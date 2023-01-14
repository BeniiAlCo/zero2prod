use std::net::TcpListener;

use axum::{http::StatusCode, routing::get, routing::IntoMakeService, Router, Server};
use hyper::server::conn::AddrIncoming;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub fn run(listener: TcpListener) -> hyper::Result<Server<AddrIncoming, IntoMakeService<Router>>> {
    let app = Router::new().route("/health_check", get(health_check));

    let server = Server::from_tcp(listener)?.serve(app.into_make_service());

    Ok(server)
}
