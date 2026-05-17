use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::post;

use crate::error::{AppError, AppResult};
use crate::mining::{MineRequest, MineResponse, pipeline};

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/mine", post(mine))
}

async fn mine(
    State(state): State<AppState>,
    Json(body): Json<MineRequest>,
) -> AppResult<Json<MineResponse>> {
    let pool = Arc::new(state.db.clone());
    let resp = pipeline::run(pool, state.config.clone(), state.jmdict.clone(), body)
        .await
        .map_err(AppError::Other)?;
    Ok(Json(resp))
}
