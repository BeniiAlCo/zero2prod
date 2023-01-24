//use secrecy::ExposeSecret;
//use std::net::TcpListener;
//use tokio_postgres::NoTls;
use axum::routing::get;
use hyper::Body;
use hyper::Request;
use hyper::StatusCode;
use tower_http::trace::TraceLayer;
use uuid::Uuid;
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    //let configuration = get_configuration().expect("Failed to read configuration.");

    //let address = "0.0.0.0:8000".to_string();

    //let manager = bb8_postgres::PostgresConnectionManager::new_from_stringlike(
    //    configuration.database.connection_string().expose_secret(),
    //    NoTls,
    //)
    //.unwrap();
    //let pool = bb8::Pool::builder()
    //    .build(manager)
    //   .await
    //   .expect("Failed to establish connection to database.");

    //let listener =
    //    TcpListener::bind(address).unwrap_or_else(|port| panic!("Failed to bind to port {port}"));

    //println!("{:?}", &listener);

    let app = axum::Router::new()
        .route("/", get(|| async { "hi" }))
        .route("/health_check", get(|| async { StatusCode::OK }))
        //.route("/subscribe", post(subscribe))
        //.with_state(connection)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                tracing::info_span!(
                    "Request",
                    request_id = %Uuid::new_v4().to_string(),
                    request_path = %request.uri(),
                )
            }),
        );

    //tracing::info!(
    //    "listening on {}",
    //    listener
    //        .local_addr()
    //        .expect("Error parsing server address.")
    //);

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    //run(listener, pool)?.await
}
