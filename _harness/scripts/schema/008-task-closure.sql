-- Recoverable terminal closure state (CL-43).
-- Existing migration files are immutable; this extends the command-first task root.
ALTER TABLE task ADD COLUMN closure_nonce TEXT;

CREATE UNIQUE INDEX IF NOT EXISTS task_closure_nonce_unique
    ON task(closure_nonce)
    WHERE closure_nonce IS NOT NULL;

INSERT INTO schema_version(version) VALUES (8);
