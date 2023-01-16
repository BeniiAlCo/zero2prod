use tokio_postgres::NoTls;
use zero2prod::{configuration::get_configuration, startup::run};

fn spawn_app() -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let address = spawn_app();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_string = configuration.database.connection_string();
    let (query, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .expect("Failed to connect to Postgres");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let client = reqwest::Client::new();

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{address}/subscribe"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = query
        .query_one(
            "SELECT $1::TEXT, $2::TEXT FROM subscriptions",
            &[&"email", &"name"],
        )
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
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing name and email"),
    ];

    // Act
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{address}/subscribe"))
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
