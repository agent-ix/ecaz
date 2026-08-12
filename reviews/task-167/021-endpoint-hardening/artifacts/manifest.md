head_sha: 48195c196
task_bucket: reviews/task-167
packet: 021-endpoint-hardening
timestamp: 2026-08-12 (America/Los_Angeles)
scope: FR-083 AC-5 physical DML endpoint security boundary
source_files: src/lib.rs; src/tests/ec_distann_basic.rs; spec/matrix/distann.md
validation: PG18 schema generation and focused pg_test

The schema command generated the extension SQL from the checkpoint and
confirmed the exact OID/bytea/array signatures. The focused test installed the
PG18 extension, invoked each physical DML endpoint as a schema-using
unprivileged role, and verified function-level permission denial. No corpus,
cluster, or operational log is part of this packet.
