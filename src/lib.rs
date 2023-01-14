use axum::{http::StatusCode, routing::get, routing::IntoMakeService, Router, Server};
use std::net::SocketAddr;

async fn health_check() -> StatusCode {
    StatusCode::OK
}

pub fn run() -> hyper::Result<Server<hyper::server::conn::AddrIncoming, IntoMakeService<Router>>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    let app = Router::new().route("/health_check", get(health_check));

    let server = Server::bind(&addr).serve(app.into_make_service());

    Ok(server)
}
