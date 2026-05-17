pub mod auth;
pub mod health;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use tower_http::trace::TraceLayer;

use crate::config::Config;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub config: Arc<Config>,
}

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        // protected routes will be merged here in later phases
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_bearer,
        ));

    let public = Router::new().merge(health::routes());

    Router::new()
        .nest("/api", public.merge(protected))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
