use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, request::Parts},
};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};

use crate::error::AppError;
use crate::state::AppState;

pub struct RequireToken {
    pub token_id: String,
    pub is_admin: bool,
}

pub struct RequireAdmin(pub RequireToken);

pub async fn validate_token(headers: &HeaderMap, db: &SqlitePool) -> Result<RequireToken, AppError> {
    let header = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("missing authorization header"))?;

    let token = header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::unauthorized("invalid authorization scheme"))?;

    let hash_bytes = Sha256::digest(token.as_bytes());
    let token_hash: String = hash_bytes.iter().map(|b| format!("{:02x}", b)).collect();

    let row = sqlx::query(
        "SELECT id, is_admin FROM tokens \
         WHERE token_hash = ? AND (expires_at IS NULL OR expires_at > datetime('now'))",
    )
    .bind(&token_hash)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::internal(e))?;

    let row = row.ok_or_else(|| AppError::unauthorized("invalid or expired token"))?;

    Ok(RequireToken {
        token_id: row.get("id"),
        is_admin: row.get("is_admin"),
    })
}

pub async fn validate_admin(
    headers: &HeaderMap,
    db: &SqlitePool,
) -> Result<RequireToken, AppError> {
    let token = validate_token(headers, db).await?;
    if !token.is_admin {
        return Err(AppError::forbidden("admin access required"));
    }
    Ok(token)
}

impl FromRequestParts<AppState> for RequireToken {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        validate_token(&parts.headers, &state.db).await
    }
}

impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        validate_admin(&parts.headers, &state.db)
            .await
            .map(RequireAdmin)
    }
}
