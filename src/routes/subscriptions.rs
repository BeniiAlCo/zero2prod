use axum::{
    extract::{Form, State},
    http::StatusCode,
};
use deadpool_postgres::Pool;
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Subscription {
    name: String,
    email: String,
}

pub async fn subscribe(
    State(connection): State<Pool>,
    Form(input): Form<Subscription>,
) -> StatusCode {
    tracing::info!("Saving new subscriber details in the database.");

    match connection
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap()
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
        .await
    {
        Ok(_) => {
            tracing::info!("New subscriber details have been saved.");
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!("Failed to execute query: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
