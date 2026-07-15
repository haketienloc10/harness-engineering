-- Canonical main-lineage migration 013: portable task/capsule projection.
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS portable_task_summary (
    task_id TEXT PRIMARY KEY,
    capsule_path TEXT NOT NULL UNIQUE,
    capsule_schema TEXT NOT NULL,
    task_date TEXT NOT NULL,
    risk_lane TEXT NOT NULL CHECK(risk_lane IN ('tiny','normal','high_risk')),
    outcome TEXT NOT NULL,
    summary TEXT NOT NULL,
    story_ids_json TEXT NOT NULL DEFAULT '[]' CHECK(json_valid(story_ids_json)),
    trace_ids_json TEXT NOT NULL DEFAULT '[]' CHECK(json_valid(trace_ids_json)),
    proof_summaries_json TEXT NOT NULL DEFAULT '[]' CHECK(json_valid(proof_summaries_json)),
    unknown_fields_json TEXT NOT NULL DEFAULT '[]' CHECK(json_valid(unknown_fields_json)),
    content_checksum TEXT NOT NULL,
    projected_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT INTO schema_version(version) VALUES (13);
