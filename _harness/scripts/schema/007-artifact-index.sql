-- Rebuildable projection of Git-tracked semantic artifacts (CL-30).
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS artifact_index (
    artifact_type TEXT NOT NULL CHECK(artifact_type IN ('story', 'decision', 'capsule')),
    artifact_id TEXT NOT NULL,
    path TEXT NOT NULL,
    checksum TEXT NOT NULL,
    schema_version TEXT NOT NULL,
    status TEXT NOT NULL,
    source_commit TEXT,
    indexed_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (artifact_type, artifact_id),
    UNIQUE (path)
);

INSERT INTO schema_version(version) VALUES (7);
