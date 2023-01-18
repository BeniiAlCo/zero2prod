use zero2prod::{configuration::get_configuration, configuration::DatabaseSettings, startup::run};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub struct TestApp {
    pub address: String,
    pub db_pool: bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>,
}

async fn spawn_app() -> TestApp {
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

pub async fn configure_database(
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

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscribe", app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = app
        .db_pool
        .get()
        .await
        .unwrap()
        .query_one("SELECT email, name FROM subscriptions", &[])
        .await
        .expect("Failed to fetch saved subscriptions");

    let saved_email: &str = saved.get("email");
    let saved_name: &str = saved.get("name");

    assert_eq!(saved_email, "ursula_le_guin@gmail.com");
    assert_eq!(saved_name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing name and email"),
    ];

    // Act
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscribe", app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with '400 Bad Request' when the payload was {error_message}."
        );
    }
}
