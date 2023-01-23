use axum::{
    extract::{Form, State},
    http::StatusCode,
};
use serde::Deserialize;
use time::OffsetDateTime;
use tracing::instrument;
use uuid::Uuid;

use crate::startup::DbPool;

#[derive(Debug, Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

#[instrument(
    name = "Adding a new subscriber",
    skip_all,
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name),
    level = "info"
)]
pub async fn subscribe(State(pool): State<DbPool>, Form(form): Form<FormData>) -> StatusCode {
    match get_connection(&pool).await {
        Ok(connection) => match insert_subscriber(&connection, &form).await {
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
        bb8_postgres::PostgresConnectionManager<postgres_native_tls::MakeTlsConnector>,
    >,
    form: &FormData,
) -> Result<(), tokio_postgres::Error> {
    connection
        .execute(
            "INSERT INTO Subscriptions (id, email, name, subscribed_at) 
    VALUES ($1, $2, $3, $4)",
            &[
                &Uuid::new_v4(),
                &form.email,
                &form.name,
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
    pool: &DbPool,
) -> Result<
    bb8::PooledConnection<
        bb8_postgres::PostgresConnectionManager<postgres_native_tls::MakeTlsConnector>,
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
