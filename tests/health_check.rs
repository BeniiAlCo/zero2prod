use zero2prod::{configuration::get_configuration, startup::run};

pub struct TestApp {
    pub address: String,
    pub db_pool: bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>,
}

async fn spawn_app() -> TestApp {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let manager = bb8_postgres::PostgresConnectionManager::new_from_stringlike(
        configuration.database.connection_string(),
        tokio_postgres::NoTls,
    )
    .unwrap();
    let pool = bb8::Pool::builder().build(manager).await.unwrap();

    let server = run(listener, pool.clone()).unwrap();
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: pool,
    }
}

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(format!("{}/health_check", app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
