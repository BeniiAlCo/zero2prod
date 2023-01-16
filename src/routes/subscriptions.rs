use axum::{
    extract::{Form, State},
    http::StatusCode,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use chrono::Utc;
use serde::Deserialize;
use tokio_postgres::NoTls;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Subscription {
    name: String,
    email: String,
}

pub async fn subscribe(
    State(connection): State<Pool<PostgresConnectionManager<NoTls>>>,
    Form(input): Form<Subscription>,
) -> StatusCode {
    match connection
        .get()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .unwrap()
        .execute(
            "INSERT INTO Subscriptions (id, email, name, subscribed_at) 
        VALUES ($1, $2, $3, $4)",
            &[&Uuid::new_v4(), &input.email, &input.name, &Utc::now()],
        )
        .await
    {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Failed to execute query: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
