use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
    Router,
    routing::{delete, post}
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::auth::RequireAdmin;
use crate::error::AppError;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/v1/tokens", post(create_token).get(list_tokens))
        .route("/v1/tokens/{id}", delete(revoke_token))
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    label: String,
    #[serde(default)]
    is_admin: bool,
    expires_at: Option<String>,
}

#[derive(Serialize)]
struct CreateTokenResponse {
    id: String,
    token: String,
    label: String,
    is_admin: bool,
}

#[derive(Serialize, sqlx::FromRow)]
struct TokenInfo {
    id: String,
    label: String,
    is_admin: bool,
    expires_at: Option<String>,
    created_at: String,
}

async fn create_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTokenRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Bootstrap: if no tokens exist, allow first token creation without auth
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM tokens")
        .fetch_one(&state.db)
        .await?;

    let is_bootstrap = count == 0;

    if !is_bootstrap {
        crate::auth::validate_admin(&headers, &state.db).await?;
    }

    let id = Uuid::new_v4().to_string();
    let token = format!("cask_{}", Uuid::new_v4());
    let hash_bytes = Sha256::digest(token.as_bytes());
    let token_hash: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    // Bootstrap token is always admin
    let is_admin = if is_bootstrap { true } else { body.is_admin };

    sqlx::query(
        "INSERT INTO tokens (id, token_hash, label, is_admin, expires_at) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&token_hash)
    .bind(&body.label)
    .bind(is_admin)
    .bind(&body.expires_at)
    .execute(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateTokenResponse {
            id,
            token,
            label: body.label,
            is_admin,
        }),
    ))
}

async fn list_tokens(
    State(state): State<AppState>,
    _auth: RequireAdmin,
) -> Result<Json<Vec<TokenInfo>>, AppError> {
    let rows = sqlx::query_as::<_, TokenInfo>(
        "SELECT id, label, is_admin, expires_at, created_at \
         FROM tokens ORDER BY created_at DESC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}

async fn revoke_token(
    State(state): State<AppState>,
    _auth: RequireAdmin,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let result = sqlx::query("DELETE FROM tokens WHERE id = ?")
        .bind(&id)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found("token not found"));
    }

    Ok(StatusCode::NO_CONTENT)
}
