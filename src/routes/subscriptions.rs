use axum::{
    extract::{Form, State},
    http::StatusCode,
};
use serde::Deserialize;
use time::OffsetDateTime;
use tracing::instrument;
use tracing::Instrument;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Subscription {
    name: String,
    email: String,
}

#[instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        request_id = %Uuid::new_v4(),
        subscriber_email = %input.email, 
        subscriber_name = %input.name), 
    level = "info"
)]
pub async fn subscribe(
    State(connection): State<
        bb8::Pool<bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>,
    >,
    Form(input): Form<Subscription>,
) -> StatusCode {
    let connection_span =
        tracing::span!(tracing::Level::INFO, "Getting connection to the database.");

    let query_span = tracing::span!(
        tracing::Level::INFO,
        "Saving new subscriber details in the database."
    );

    let conn = connection
        .get()
        .in_current_span()
        .instrument(connection_span)
        .await
        .expect("Failed to establish connection to database.");

    match conn
        .execute(
            "INSERT INTO Subscriptions (id, email, name, subscribed_at) 
        VALUES ($1, $2, $3, $4)",
            &[
                &Uuid::new_v4(),
                &input.email,
                &input.name,
                &OffsetDateTime::now_utc(),
            ],
        )
        .instrument(query_span)
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
