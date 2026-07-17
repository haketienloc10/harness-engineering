-- Canonical main-lineage migration 011: complete structured proof provenance.
PRAGMA foreign_keys = ON;

ALTER TABLE proof_run ADD COLUMN story_id TEXT REFERENCES story(id);
ALTER TABLE proof_run ADD COLUMN shell_mode INTEGER NOT NULL DEFAULT 0 CHECK(shell_mode IN (0,1));
ALTER TABLE proof_run ADD COLUMN cwd TEXT;
ALTER TABLE proof_run ADD COLUMN branch TEXT;
ALTER TABLE proof_run ADD COLUMN dirty_fingerprint TEXT;
ALTER TABLE proof_run ADD COLUMN cli_version TEXT;
ALTER TABLE proof_run ADD COLUMN platform TEXT;
ALTER TABLE proof_run ADD COLUMN command_digest TEXT;
ALTER TABLE proof_run ADD COLUMN stdout_path TEXT;
ALTER TABLE proof_run ADD COLUMN stdout_hash TEXT;
ALTER TABLE proof_run ADD COLUMN stderr_path TEXT;
ALTER TABLE proof_run ADD COLUMN stderr_hash TEXT;
ALTER TABLE proof_run ADD COLUMN artifact_path TEXT;
ALTER TABLE proof_run ADD COLUMN artifact_hash TEXT;

INSERT INTO schema_version(version) VALUES (11);
