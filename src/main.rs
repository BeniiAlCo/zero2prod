use openssl::ssl::{SslConnector, SslMethod};
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

    let connector = MakeTlsConnector::new(SslConnector::builder(SslMethod::tls()).unwrap().build());

    let (client, connection) = configuration
        .database
        .with_db()
        .connect(connector.to_owned())
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client.query("SELECT $1::TEXT", &[&"name"]).await.unwrap();
    let value: &str = rows[0].get(0);
    dbg!(value);

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
