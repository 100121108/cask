use axum::extract::{Path, State};
use axum::{Json, Router, routing::get};
use serde::Serialize;

use crate::error::AppError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/artifacts/{name}/{version}/stats",
            get(version_stats),
        )
        .route("/v1/artifacts/{name}/stats", get(artifact_stats))
}

#[derive(Serialize)]
struct StatsResponse {
    downloads: i64,
}

async fn version_stats(
    State(state): State<AppState>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Json<StatsResponse>, AppError> {
    let downloads = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM download_stats \
         WHERE artifact_id = (SELECT id FROM artifacts WHERE name = ? AND version = ?)",
    )
    .bind(&name)
    .bind(&version)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(StatsResponse { downloads }))
}

async fn artifact_stats(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<StatsResponse>, AppError> {
    let downloads = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM download_stats ds \
         JOIN artifacts a ON ds.artifact_id = a.id WHERE a.name = ?",
    )
    .bind(&name)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(StatsResponse { downloads }))
}
