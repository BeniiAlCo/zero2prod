use crate::routes::{health_check, subscribe};
use axum::{routing, routing::get, routing::post};
use hyper::server::conn;
use std::net::TcpListener;

pub fn run(
    listener: TcpListener,
    connection: bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>,
) -> hyper::Result<axum::Server<conn::AddrIncoming, routing::IntoMakeService<axum::Router>>> {
    let app = axum::Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .with_state(connection);

    tracing::info!(
        "listening on {:?}",
        listener
            .local_addr()
            .expect("Error extracting local address from TCP listener.")
    );
    let server = axum::Server::from_tcp(listener)?.serve(app.into_make_service());

    Ok(server)
}
