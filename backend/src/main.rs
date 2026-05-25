use tokio::net::TcpListener;

mod app;
mod config;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = config::AppConfig::from_env();
    let bind_address = config.bind_address();
    let app = app::router();

    let listener = TcpListener::bind(&bind_address)
        .await
        .expect("failed to bind backend address");

    tracing::info!("rankr backend listening on http://{}", bind_address);

    axum::serve(listener, app)
        .await
        .expect("backend server failed");
}
