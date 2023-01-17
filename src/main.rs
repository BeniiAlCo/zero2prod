use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use std::net::TcpListener;
use tokio_postgres::NoTls;
use zero2prod::configuration::get_configuration;
use zero2prod::startup::run;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!("127.0.0.1:{}", configuration.application_port);

    let mut pool_cfg = Config::new();
    pool_cfg.host = Some(configuration.database.host.to_string());
    pool_cfg.user = Some(configuration.database.username.to_string());
    pool_cfg.password = Some(configuration.database.password.to_string());
    pool_cfg.dbname = Some(configuration.database.database_name.to_string());
    pool_cfg.port = Some(configuration.database.port);

    pool_cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = pool_cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    let listener =
        TcpListener::bind(address).unwrap_or_else(|port| panic!("Failed to bind to port {port}"));

    run(listener, pool)?.await
}
