use axum::{Json, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

pub async fn health() -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok",
        service: "rankr_backend",
    })
}
