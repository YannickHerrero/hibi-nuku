pub mod auth;
pub mod health;
pub mod library;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::tokenize::jmdict_index::JmdictIndex;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
    pub jmdict: Arc<JmdictIndex>,
}

pub fn router(state: AppState) -> Router {
    let public = Router::new().merge(health::routes());

    let protected = Router::new().merge(library::routes()).route_layer(
        middleware::from_fn_with_state(state.clone(), auth::require_bearer),
    );

    Router::new()
        .nest("/api", public.merge(protected))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
