pub mod auth;
pub mod dict;
pub mod health;
pub mod hibi_proxy;
pub mod library;
pub mod mine;
pub mod settings;
pub mod upload;
pub mod videos;

use std::sync::Arc;

use axum::Router;
use axum::middleware;
use sqlx::SqlitePool;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::tokenize::jmdict_index::JmdictIndex;

use self::hibi_proxy::KnownCache;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: SqlitePool,
    pub jmdict: Arc<JmdictIndex>,
    pub known_cache: Arc<KnownCache>,
}

pub fn router(state: AppState) -> Router {
    let public = Router::new().merge(health::routes());

    let protected = Router::new()
        .merge(library::routes())
        .merge(upload::routes())
        .merge(videos::routes())
        .merge(dict::routes())
        .merge(mine::routes())
        .merge(hibi_proxy::routes())
        .merge(settings::routes())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_bearer,
        ));

    let mut app = Router::new()
        .nest("/api", public.merge(protected))
        .with_state(state.clone());

    // Production: serve the built frontend from frontend/dist. In dev
    // Vite proxies /api → backend, so the static fallback never runs.
    let dist = std::path::PathBuf::from("frontend/dist");
    if dist.exists() {
        let index = dist.join("index.html");
        let serve_dir = ServeDir::new(&dist).fallback(ServeFile::new(&index));
        app = app.fallback_service(serve_dir);
    }

    app.layer(TraceLayer::new_for_http())
}
