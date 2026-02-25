use axum::body::Bytes;
use axum::extract::{ConnectInfo, Path, Query, State};
use axum::http::{HeaderMap, header};
use axum::response::IntoResponse;
use axum::{Json, Router, routing::{get, put}};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::RequireToken;
use crate::error::AppError;
use crate::state::AppState;
use crate::storage;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/artifacts", get(list_artifacts))
        .route("/v1/artifacts/{name}", get(list_versions))
        .route(
            "/v1/artifacts/{name}/{version}",
            put(upload).get(download).delete(delete_artifact),
        )
}

#[derive(Serialize, sqlx::FromRow)]
struct ArtifactRow {
    id: String,
    name: String,
    version: String,
    filename: String,
    sha256: String,
    size: i64,
    created_at: String,
}

#[derive(Deserialize)]
struct UploadParams {
    filename: Option<String>,
}

async fn list_artifacts(
    State(state): State<AppState>,
) -> Result<Json<Vec<ArtifactRow>>, AppError> {
    let rows = sqlx::query_as::<_, ArtifactRow>(
        "SELECT id, name, version, filename, sha256, size, created_at \
         FROM artifacts ORDER BY name, created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

async fn list_versions(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Vec<ArtifactRow>>, AppError> {
    let rows = sqlx::query_as::<_, ArtifactRow>(
        "SELECT id, name, version, filename, sha256, size, created_at \
         FROM artifacts WHERE name = ? ORDER BY created_at DESC",
    )
    .bind(&name)
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

async fn upload(
    State(state): State<AppState>,
    _auth: RequireToken,
    Path((name, version)): Path<(String, String)>,
    Query(params): Query<UploadParams>,
    body: Bytes,
) -> Result<impl IntoResponse, AppError> {
    if body.len() > state.max_upload_size {
        return Err(AppError::payload_too_large(format!(
            "upload size {} exceeds maximum {}",
            body.len(),
            state.max_upload_size
        )));
    }

    // Check for duplicate
    let existing = sqlx::query("SELECT id FROM artifacts WHERE name = ? AND version = ?")
        .bind(&name)
        .bind(&version)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::conflict(format!(
            "artifact {}/{} already exists",
            name, version
        )));
    }

    let filename = params
        .filename
        .unwrap_or_else(|| format!("{}-{}", name, version));
    let id = Uuid::new_v4().to_string();
    let hash_bytes = Sha256::digest(&body);
    let sha256: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();
    let size = body.len() as i64;

    storage::save(&state.data_dir, &id, &body)
        .await
        .map_err(anyhow::Error::from)?;

    sqlx::query(
        "INSERT INTO artifacts (id, name, version, filename, sha256, size) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&name)
    .bind(&version)
    .bind(&filename)
    .bind(&sha256)
    .bind(size)
    .execute(&state.db)
    .await?;

    let artifact = sqlx::query_as::<_, ArtifactRow>(
        "SELECT id, name, version, filename, sha256, size, created_at \
         FROM artifacts WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(&state.db)
    .await?;

    Ok((axum::http::StatusCode::CREATED, Json(artifact)))
}

async fn download(
    State(state): State<AppState>,
    Path((name, version)): Path<(String, String)>,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
) -> Result<impl IntoResponse, AppError> {
    let artifact = sqlx::query_as::<_, ArtifactRow>(
        "SELECT id, name, version, filename, sha256, size, created_at \
         FROM artifacts WHERE name = ? AND version = ?",
    )
    .bind(&name)
    .bind(&version)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::not_found(format!("artifact {}/{} not found", name, version)))?;

    // Record download stat
    let stat_id = Uuid::new_v4().to_string();
    let ip = addr.ip().to_string();
    let _ = sqlx::query("INSERT INTO download_stats (id, artifact_id, ip) VALUES (?, ?, ?)")
        .bind(&stat_id)
        .bind(&artifact.id)
        .bind(&ip)
        .execute(&state.db)
        .await;

    let bytes = storage::load(&state.data_dir, &artifact.id)
        .await
        .map_err(anyhow::Error::from)?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/octet-stream".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}\"", artifact.filename)
            .parse()
            .unwrap(),
    );

    Ok((headers, bytes))
}

async fn delete_artifact(
    State(state): State<AppState>,
    _auth: RequireToken,
    Path((name, version)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let artifact = sqlx::query_as::<_, ArtifactRow>(
        "SELECT id, name, version, filename, sha256, size, created_at \
         FROM artifacts WHERE name = ? AND version = ?",
    )
    .bind(&name)
    .bind(&version)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::not_found(format!("artifact {}/{} not found", name, version)))?;

    // Delete from DB (cascades to metadata + stats)
    sqlx::query("DELETE FROM artifacts WHERE id = ?")
        .bind(&artifact.id)
        .execute(&state.db)
        .await?;

    // Delete file from disk
    storage::delete(&state.data_dir, &artifact.id)
        .await
        .map_err(anyhow::Error::from)?;

    Ok(axum::http::StatusCode::NO_CONTENT)
}
