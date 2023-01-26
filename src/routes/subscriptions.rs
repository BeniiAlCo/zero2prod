use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use axum::{
    extract::{Form, State},
    http::StatusCode,
};
use serde::Deserialize;
use time::OffsetDateTime;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}

#[instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name),
    level = "info"
)]
pub async fn subscribe(
    State(pool): State<
        bb8::Pool<bb8_postgres::PostgresConnectionManager<postgres_openssl::MakeTlsConnector>>,
    >,
    Form(form): Form<FormData>,
) -> StatusCode {
    let new_subscriber = match form.try_into() {
        Ok(valid_subscriber) => valid_subscriber,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    match get_connection(&pool).await {
        Ok(connection) => match insert_subscriber(&connection, &new_subscriber).await {
            Ok(_) => StatusCode::OK,
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
        },
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[instrument(name = "Saving new subscriber details in the database", skip_all)]
async fn insert_subscriber(
    connection: &bb8::PooledConnection<
        '_,
        bb8_postgres::PostgresConnectionManager<postgres_openssl::MakeTlsConnector>,
    >,
    new_subscriber: &NewSubscriber,
) -> Result<(), tokio_postgres::Error> {
    connection
        .execute(
            "INSERT INTO Subscriptions (id, email, name, subscribed_at) 
    VALUES ($1, $2, $3, $4)",
            &[
                &Uuid::new_v4(),
                &new_subscriber.email.as_ref(),
                &new_subscriber.name.as_ref(),
                &OffsetDateTime::now_utc(),
            ],
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to execute query: {:?}", e);
            e
        })?;

    Ok(())
}

#[instrument(
    name = "Establishing new connection to the database"
    skip_all,
)]
async fn get_connection(
    pool: &bb8::Pool<bb8_postgres::PostgresConnectionManager<postgres_openssl::MakeTlsConnector>>,
) -> Result<
    bb8::PooledConnection<
        bb8_postgres::PostgresConnectionManager<postgres_openssl::MakeTlsConnector>,
    >,
    bb8::RunError<tokio_postgres::Error>,
> {
    pool.get().await.map_err(|e| {
        tracing::error!(
            "Failed to get connection to database from connection pool: {:?}",
            e
        );
        e
    })
}
