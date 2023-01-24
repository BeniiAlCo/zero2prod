use crate::routes::{health_check, subscribe};
use axum::{routing, routing::get, routing::post};
use hyper::{server::conn, Body, Request};
use std::net::TcpListener;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

pub async fn run(
    listener: TcpListener,
    connection: bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>,
) -> hyper::Result<axum::Server<conn::AddrIncoming, routing::IntoMakeService<axum::Router>>> {
    let app = axum::Router::new()
        .route("/health_check", get(health_check))
        .route("/subscribe", post(subscribe))
        .with_state(connection)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                tracing::info_span!(
                    "Request",
                    request_id = %Uuid::new_v4().to_string(),
                    request_path = %request.uri(),
                )
            }),
        );

    tracing::info!(
        "listening on {}",
        listener
            .local_addr()
            .expect("Error parsing server address.")
    );
    let server =
        axum::Server::from_tcp(listener.try_clone().unwrap())?.serve(app.into_make_service());

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", listener.local_addr().unwrap()))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());

    Ok(server)
}
