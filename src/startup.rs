use axum::{routing, routing::get, routing::post};
use hyper::server::conn;
use std::net::TcpListener;
use tower_http::trace;
use tracing::Level;

use crate::routes::{health_check, subscribe};

pub fn run(
    listener: TcpListener,
    connection: deadpool_postgres::Pool,
) -> hyper::Result<axum::Server<conn::AddrIncoming, routing::IntoMakeService<axum::Router>>> {
    tracing_subscriber::fmt().init();

    let app = axum::Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .with_state(connection)
        .layer(
            trace::TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        );

    tracing::info!(
        "listening on {:?}",
        listener
            .local_addr()
            .expect("Error extracting local address from TCP listener.")
    );
    let server = axum::Server::from_tcp(listener)?.serve(app.into_make_service());

    Ok(server)
}
