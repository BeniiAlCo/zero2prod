use std::net::TcpListener;
use tokio_postgres::NoTls;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);
    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber.");

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);

    let manager = bb8_postgres::PostgresConnectionManager::new_from_stringlike(
        configuration.database.connection_string(),
        NoTls,
    )
    .unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();

    let listener =
        TcpListener::bind(address).unwrap_or_else(|port| panic!("Failed to bind to port {port}"));

    run(listener, pool)?.await
}
