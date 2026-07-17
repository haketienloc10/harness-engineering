# CL-31 Design

Canonical artifact parsing must yield typed IDs, paths, status, lane and
checksums. The rebuild service initializes a temp database using the canonical
migration manifest, inserts projections in one transaction, runs doctor/audit
against that temp path, and returns a conflict/provenance report. It never uses
the current DB as semantic input.
