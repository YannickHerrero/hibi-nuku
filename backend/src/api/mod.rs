pub mod auth;
pub mod health;

use std::sync::Arc;

use axum::Router;
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;

use crate::config::Config;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
}

pub fn router(state: AppState) -> Router {
    let public = Router::new().merge(health::routes());

    // `protected` will be merged with .route_layer(auth::require_bearer)
    // once Phase 2 adds the first protected route.
    let protected: Router<AppState> = Router::new();

    Router::new()
        .nest("/api", public.merge(protected))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
