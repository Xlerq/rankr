use tokio::net::TcpListener;

mod app;
mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = app::router();

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind backend address");

    tracing::info!("rankr backend listening on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("backend server failed");
}
