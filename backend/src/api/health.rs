use axum::Json;
use axum::Router;
use axum::routing::get;
use serde::Serialize;

use super::AppState;

#[derive(Serialize)]
struct Health {
    ok: bool,
    version: &'static str,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<Health> {
    Json(Health {
        ok: true,
        version: env!("CARGO_PKG_VERSION"),
    })
}
