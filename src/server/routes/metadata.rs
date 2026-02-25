use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router, routing::{delete, get}};
use sqlx::Row;

use crate::auth::RequireToken;
use crate::error::AppError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/artifacts/{name}/{version}/meta",
            get(get_metadata).put(set_metadata),
        )
        .route(
            "/v1/artifacts/{name}/{version}/meta/{key}",
            delete(delete_metadata),
        )
}

async fn get_metadata(
    State(state): State<AppState>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Json<HashMap<String, String>>, AppError> {
    let artifact_id = lookup_artifact_id(&state, &name, &version).await?;

    let rows = sqlx::query("SELECT key, value FROM artifact_metadata WHERE artifact_id = ?")
        .bind(&artifact_id)
        .fetch_all(&state.db)
        .await?;

    let meta: HashMap<String, String> = rows
        .iter()
        .map(|r| (r.get("key"), r.get("value")))
        .collect();

    Ok(Json(meta))
}

async fn set_metadata(
    State(state): State<AppState>,
    _auth: RequireToken,
    Path((name, version)): Path<(String, String)>,
    Json(body): Json<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let artifact_id = lookup_artifact_id(&state, &name, &version).await?;

    for (key, value) in &body {
        sqlx::query(
            "INSERT OR REPLACE INTO artifact_metadata (artifact_id, key, value) \
             VALUES (?, ?, ?)",
        )
        .bind(&artifact_id)
        .bind(key)
        .bind(value)
        .execute(&state.db)
        .await?;
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn delete_metadata(
    State(state): State<AppState>,
    _auth: RequireToken,
    Path((name, version, key)): Path<(String, String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let artifact_id = lookup_artifact_id(&state, &name, &version).await?;

    sqlx::query("DELETE FROM artifact_metadata WHERE artifact_id = ? AND key = ?")
        .bind(&artifact_id)
        .bind(&key)
        .execute(&state.db)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

async fn lookup_artifact_id(
    state: &AppState,
    name: &str,
    version: &str,
) -> Result<String, AppError> {
    let row = sqlx::query("SELECT id FROM artifacts WHERE name = ? AND version = ?")
        .bind(name)
        .bind(version)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| {
            AppError::not_found(format!("artifact {}/{} not found", name, version))
        })?;

    Ok(row.get("id"))
}
