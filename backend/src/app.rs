use axum::Router;

use crate::routes;

pub fn router() -> Router {
    Router::new().nest("/api", routes::router())
}
