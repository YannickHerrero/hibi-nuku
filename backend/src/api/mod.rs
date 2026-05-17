pub mod health;

use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;

use crate::config::Config;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub config: Arc<Config>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .nest("/api", api_router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}

fn api_router() -> Router<AppState> {
    Router::new().merge(health::routes())
}
