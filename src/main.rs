use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use std::net::TcpListener;
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

    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.set_verify(SslVerifyMode::NONE);
    let connector = MakeTlsConnector::new(builder.build());

    let manager =
        bb8_postgres::PostgresConnectionManager::new(configuration.database.with_db(), connector);

    let pool = bb8::Pool::builder()
        .connection_timeout(std::time::Duration::from_secs(10))
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
