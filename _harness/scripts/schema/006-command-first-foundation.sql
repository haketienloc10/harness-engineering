-- Canonical main-lineage migration 006: command-first operational foundation.
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS harness_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS migration_history (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    cli_version TEXT,
    source_commit TEXT
);

INSERT INTO harness_meta(key, value) VALUES ('schema_lineage', 'main')
ON CONFLICT(key) DO UPDATE SET value=excluded.value;

CREATE TABLE IF NOT EXISTS task (
    id TEXT PRIMARY KEY,
    intake_id INTEGER UNIQUE REFERENCES intake(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    closed_at TEXT,
    status TEXT NOT NULL CHECK(status IN ('open','in_progress','blocked','closing','completed','abandoned','failed')),
    outcome TEXT,
    risk_lane TEXT NOT NULL CHECK(risk_lane IN ('tiny','normal','high_risk')),
    behavior_bearing INTEGER NOT NULL CHECK(behavior_bearing IN (0,1)),
    summary TEXT NOT NULL,
    owner TEXT,
    worktree TEXT NOT NULL,
    branch TEXT,
    start_commit TEXT,
    end_commit TEXT,
    context_manifest_json TEXT NOT NULL,
    context_manifest_checksum TEXT NOT NULL,
    capsule_required INTEGER NOT NULL CHECK(capsule_required IN (0,1)),
    capsule_path TEXT,
    capsule_checksum TEXT,
    capsule_omission_reason TEXT,
    CHECK((status IN ('completed','abandoned','failed')) = (outcome IS NOT NULL))
);

CREATE TABLE IF NOT EXISTS task_story (task_id TEXT NOT NULL REFERENCES task(id) ON DELETE CASCADE, story_id TEXT NOT NULL REFERENCES story(id), role TEXT NOT NULL DEFAULT 'primary', PRIMARY KEY(task_id, story_id));
CREATE TABLE IF NOT EXISTS task_approval (task_id TEXT NOT NULL REFERENCES task(id) ON DELETE CASCADE, gate TEXT NOT NULL, source TEXT NOT NULL, evidence TEXT NOT NULL, scope TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')));
CREATE TABLE IF NOT EXISTS task_context_read (task_id TEXT NOT NULL REFERENCES task(id) ON DELETE CASCADE, path TEXT NOT NULL, actor TEXT, created_at TEXT NOT NULL DEFAULT (datetime('now')), PRIMARY KEY(task_id, path));
CREATE TABLE IF NOT EXISTS proof_run (id INTEGER PRIMARY KEY AUTOINCREMENT, task_id TEXT NOT NULL REFERENCES task(id) ON DELETE CASCADE, layer TEXT NOT NULL, state TEXT NOT NULL CHECK(state IN ('pass','fail','not_applicable')), executable TEXT, argv_json TEXT, started_at TEXT NOT NULL DEFAULT (datetime('now')), finished_at TEXT, exit_code INTEGER, head_commit TEXT, summary TEXT);

INSERT INTO schema_version(version) VALUES (6);
