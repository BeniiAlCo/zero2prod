use axum::{extract::Form, http::StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Subscription {
    name: String,
    email: String,
}

pub async fn subscribe(Form(_input): Form<Subscription>) -> StatusCode {
    StatusCode::OK
}
