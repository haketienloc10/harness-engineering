-- Canonical main-lineage migration 012: explicit, approved audit dispositions.
PRAGMA foreign_keys = ON;

CREATE TABLE audit_disposition (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    finding_key         TEXT NOT NULL CHECK(finding_key IN (
                            'terminal_task_without_trace',
                            'unrooted_trace'
                        )),
    entity_id           TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'accepted'
                        CHECK(status IN ('accepted','revoked')),
    rationale           TEXT NOT NULL,
    provenance          TEXT NOT NULL,
    approval_task_id    TEXT NOT NULL REFERENCES task(id),
    approval_source     TEXT NOT NULL,
    actor               TEXT NOT NULL,
    created_at          TEXT NOT NULL DEFAULT (datetime('now')),
    expires_at          TEXT,
    revoked_at          TEXT,
    revoked_by          TEXT,
    revocation_reason   TEXT,
    CHECK(
        (status='accepted' AND revoked_at IS NULL AND revoked_by IS NULL
                           AND revocation_reason IS NULL)
        OR
        (status='revoked' AND revoked_at IS NOT NULL AND revoked_by IS NOT NULL
                          AND revocation_reason IS NOT NULL)
    )
);

CREATE UNIQUE INDEX audit_disposition_one_accepted
    ON audit_disposition(finding_key, entity_id)
    WHERE status='accepted';

CREATE INDEX audit_disposition_approval_task
    ON audit_disposition(approval_task_id);

INSERT INTO schema_version(version) VALUES (12);
