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

    let manager = bb8_postgres::PostgresConnectionManager::new_from_stringlike(
        configuration.database.connection_string().expose_secret(),
        NoTls,
    )
    .unwrap();
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to establish connection to database.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener =
        TcpListener::bind(address).unwrap_or_else(|port| panic!("Failed to bind to port {port}"));

    run(listener, pool)?.await
}
