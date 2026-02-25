CREATE TABLE IF NOT EXISTS artifacts (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    version     TEXT NOT NULL,
    filename    TEXT NOT NULL,
    sha256      TEXT NOT NULL,
    size        INTEGER NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(name, version)
);

CREATE TABLE IF NOT EXISTS artifact_metadata (
    artifact_id TEXT NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    key         TEXT NOT NULL,
    value       TEXT NOT NULL,
    PRIMARY KEY (artifact_id, key)
);

CREATE TABLE IF NOT EXISTS tokens (
    id          TEXT PRIMARY KEY,
    token_hash  TEXT NOT NULL UNIQUE,
    label       TEXT NOT NULL,
    is_admin    INTEGER NOT NULL DEFAULT 0,
    expires_at  TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS download_stats (
    id            TEXT PRIMARY KEY,
    artifact_id   TEXT NOT NULL REFERENCES artifacts(id) ON DELETE CASCADE,
    downloaded_at TEXT NOT NULL DEFAULT (datetime('now')),
    ip            TEXT
);

CREATE INDEX idx_artifacts_name ON artifacts(name);
CREATE INDEX idx_download_stats_artifact ON download_stats(artifact_id);
CREATE INDEX idx_tokens_expires ON tokens(expires_at) WHERE expires_at IS NOT NULL;
