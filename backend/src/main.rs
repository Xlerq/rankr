use axum::{Json, Router, routing::get};
use serde::Serialize;
use tokio::net::TcpListener;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "rankr_backend",
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/api/health", get(health));

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind backend address");

    tracing::info!("rankr backend listening on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("backend server failed");
}
