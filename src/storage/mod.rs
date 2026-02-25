use std::path::Path;

use anyhow::{Context, Result};
use tokio::fs;

/// Save artifact bytes to disk. Returns the path written.
pub async fn save(data_dir: &Path, artifact_id: &str, bytes: &[u8]) -> Result<()> {
    let path = data_dir.join("artifacts").join(artifact_id);
    fs::write(&path, bytes)
        .await
        .with_context(|| format!("failed to write artifact to {}", path.display()))?;
    Ok(())
}

/// Load artifact bytes from disk.
pub async fn load(data_dir: &Path, artifact_id: &str) -> Result<Vec<u8>> {
    let path = data_dir.join("artifacts").join(artifact_id);
    let bytes = fs::read(&path)
        .await
        .with_context(|| format!("failed to read artifact from {}", path.display()))?;
    Ok(bytes)
}

/// Delete artifact from disk.
pub async fn delete(data_dir: &Path, artifact_id: &str) -> Result<()> {
    let path = data_dir.join("artifacts").join(artifact_id);
    if path.exists() {
        fs::remove_file(&path)
            .await
            .with_context(|| format!("failed to delete artifact at {}", path.display()))?;
    }
    Ok(())
}
