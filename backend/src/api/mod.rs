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

    // Production: serve the built frontend from frontend/dist.
    //
    // CWD when running via `cargo run` is the crate dir (`backend/`),
    // so we check a few candidate locations + honour an explicit
    // `NUKU_FRONTEND_DIST` env override.
    if let Some(dist) = find_frontend_dist() {
        let index = dist.join("index.html");
        tracing::info!(path = %dist.display(), "serving frontend dist");
        let serve_dir = ServeDir::new(&dist).fallback(ServeFile::new(&index));
        app = app.fallback_service(serve_dir);
    } else {
        tracing::warn!("frontend dist not found — running API-only");
    }

    app.layer(TraceLayer::new_for_http())
}

fn find_frontend_dist() -> Option<std::path::PathBuf> {
    if let Ok(p) = std::env::var("NUKU_FRONTEND_DIST") {
        let p = std::path::PathBuf::from(p);
        if p.join("index.html").exists() {
            return Some(p);
        }
    }
    for candidate in ["frontend/dist", "../frontend/dist", "./dist"] {
        let p = std::path::PathBuf::from(candidate);
        if p.join("index.html").exists() {
            return Some(p);
        }
    }
    None
}
