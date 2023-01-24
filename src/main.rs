use secrecy::ExposeSecret;
use std::net::TcpListener;
use tokio_postgres::NoTls;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let manager = bb8_postgres::PostgresConnectionManager::new_from_stringlike(
        configuration.database.connection_string().expose_secret(),
        NoTls,
    )
    .unwrap();
    let pool = bb8::Pool::builder()
        .connection_timeout(std::time::Duration::from_secs(2))
        .build(manager)
        .await
        .expect("Failed to establish connection to database.");

    let listener =
        TcpListener::bind(address).unwrap_or_else(|port| panic!("Failed to bind to port {port}"));

    println!("{:?}", &listener);

    let x = run(listener.try_clone().unwrap(), pool)?.await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", listener.local_addr().unwrap()))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    x
}
