use axum::{routing::get, routing::post, routing::IntoMakeService, Router, Server};
use deadpool_postgres::Pool;
use hyper::server::conn::AddrIncoming;
use std::net::TcpListener;

use crate::routes::{health_check, subscribe};

pub fn run(
    listener: TcpListener,
    connection: Pool,
) -> hyper::Result<Server<AddrIncoming, IntoMakeService<Router>>> {
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .with_state(connection);

    let server = Server::from_tcp(listener)?.serve(app.into_make_service());

    Ok(server)
}
