use sqlx::{Connection, PgConnection};
use std::sync::Once;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

pub static TRACING: Once = Once::new();
pub type DbPool =
    bb8::Pool<bb8_postgres::PostgresConnectionManager<postgres_native_tls::MakeTlsConnector>>;

pub fn tracing_init() {
    TRACING.call_once(|| {
        let default_filter_level = "info".to_string();
        let subscriber_name = "test".to_string();
        if std::env::var("TEST_LOG").is_ok() {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
            init_subscriber(subscriber);
        } else {
            let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
            init_subscriber(subscriber);
        }
    });
}

pub struct TestApp {
    pub address: String,
    pub db_pool: DbPool,
}

pub async fn spawn_app() -> TestApp {
    tracing_init();

    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = uuid::Uuid::new_v4().to_string();

    let pool = configure_database(&configuration.database).await;

    let server = run(listener, pool.clone()).unwrap();
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> DbPool {
    {
        let pool = bb8::Pool::builder()
            .build(config.without_db())
            .await
            .expect("Failed to create connection pool.");

        pool.dedicated_connection()
            .await
            .unwrap()
            .execute(
                format!(r#"CREATE DATABASE "{}";"#, &config.database_name).as_str(),
                &[],
            )
            .await
            .expect("Failed to create a database.");
    }

    // TODO: Replace with refinery migrate when the pull request for ssl functionality (#260) is
    // accepted.
    {
        sqlx::migrate!("./migrations")
            .run(
                &mut PgConnection::connect(dbg!(&config.connection_string()))
                    .await
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    let pool = bb8::Pool::builder()
        .build(config.with_db())
        .await
        .expect("Failed to create connection pool.");

    pool
}
