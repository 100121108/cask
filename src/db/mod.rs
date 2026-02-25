use std::path::Path;

use anyhow::{Context, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

pub async fn create_pool(data_dir: &Path) -> Result<SqlitePool> {
    let db_path = data_dir.join("cask.db");
    let url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .with_context(|| format!("failed to connect to database at {}", db_path.display()))?;

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await
        .context("failed to set journal mode")?;

    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await
        .context("failed to enable foreign keys")?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to run database migrations")?;

    Ok(pool)
}
