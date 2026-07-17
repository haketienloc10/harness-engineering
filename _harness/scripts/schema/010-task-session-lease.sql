-- Canonical main-lineage migration 010: task session identity and renewable leases.
PRAGMA foreign_keys = ON;

ALTER TABLE task ADD COLUMN session_id TEXT;
ALTER TABLE task ADD COLUMN lease_expires_at TEXT;

CREATE INDEX IF NOT EXISTS idx_task_active_session
    ON task(session_id, status, lease_expires_at);
CREATE INDEX IF NOT EXISTS idx_task_active_worktree
    ON task(worktree, status, lease_expires_at);

CREATE TRIGGER task_identity_insert_guard
BEFORE INSERT ON task
WHEN (NEW.owner IS NULL) <> (NEW.session_id IS NULL)
  OR (NEW.session_id IS NOT NULL AND length(trim(NEW.session_id)) = 0)
  OR (NEW.session_id IS NULL AND NEW.lease_expires_at IS NOT NULL)
  OR (NEW.session_id IS NOT NULL AND NEW.lease_expires_at IS NULL)
BEGIN
    SELECT RAISE(ABORT, 'task owner, session_id, and lease_expires_at must be supplied together');
END;

INSERT INTO schema_version(version) VALUES (10);
