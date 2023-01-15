use axum::{routing::get, routing::post, routing::IntoMakeService, Router, Server};
use hyper::server::conn::AddrIncoming;
use std::net::TcpListener;

use crate::routes::{health_check, subscribe};

pub fn run(listener: TcpListener) -> hyper::Result<Server<AddrIncoming, IntoMakeService<Router>>> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe));

    let server = Server::from_tcp(listener)?.serve(app.into_make_service());

    Ok(server)
}
