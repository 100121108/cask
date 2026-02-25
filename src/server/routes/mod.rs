mod artifacts;
mod metadata;
mod stats;
mod tokens;

use axum::{
    extract::DefaultBodyLimit,
    http::StatusCode,
    response::IntoResponse,
    Router,
    routing::get,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let max_upload = state.max_upload_size;

    Router::new()
        .route("/health", get(health))
        .merge(artifacts::routes())
        .merge(metadata::routes())
        .merge(tokens::routes())
        .merge(stats::routes())
        .layer(DefaultBodyLimit::max(max_upload))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}
