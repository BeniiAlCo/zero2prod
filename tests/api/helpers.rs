use std::sync::Once;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

pub static TRACING: Once = Once::new();

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
    pub db_pool: bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>,
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

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

async fn configure_database(
    config: &DatabaseSettings,
) -> bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>> {
    {
        let (client, connection) = tokio_postgres::connect(
            &config.connection_string_without_db(),
            tokio_postgres::NoTls,
        )
        .await
        .expect("Failed to connect to database without using a name.");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        client
            .execute(
                format!(r#"CREATE DATABASE "{}";"#, &config.database_name).as_str(),
                &[],
            )
            .await
            .expect("Failetd to create a database.");
    }

    {
        let (mut client, connection) =
            tokio_postgres::connect(&config.connection_string(), tokio_postgres::NoTls)
                .await
                .unwrap();

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        embedded::migrations::runner()
            .run_async(&mut client)
            .await
            .unwrap();

        println!("DB migrations finished!");
    }

    let manager = bb8_postgres::PostgresConnectionManager::new_from_stringlike(
        config.connection_string(),
        tokio_postgres::NoTls,
    )
    .expect("Failed to connect to Postgres.");
    let pool = bb8::Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create connection pool.");

    pool
}
