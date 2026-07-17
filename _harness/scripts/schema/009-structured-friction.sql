-- Structured friction lifecycle (CL-60).
CREATE TABLE friction (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT REFERENCES task(id) ON DELETE SET NULL,
    fingerprint TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    severity TEXT NOT NULL CHECK(severity IN ('low','medium','high','critical')),
    summary TEXT NOT NULL,
    disposition TEXT NOT NULL CHECK(disposition IN ('fixed-now','backlog','accepted-risk','not-friction')),
    status TEXT NOT NULL CHECK(status IN ('proposed','accepted','in_progress','implemented_pending_observation','validated','ineffective','reverted')),
    baseline TEXT,
    predicted_metric TEXT,
    observation_window TEXT,
    actual_outcome TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);
CREATE INDEX friction_task_status_idx ON friction(task_id, status);
INSERT INTO schema_version(version) VALUES (9);
