# Task 229 packet 003 artifact manifest

- Head SHA under validation: `6e4cb3c4a` (the seq-08 uncovered-path
  fail-closed correction follows source head `8b4618ca5`; the focused logs
  validate the preceding read/DML source, and the correction is the reviewed
  one-branch restoration of the prior uncovered behavior).
- Task bucket: `reviews/task-229/003-correctness-and-dml/`.
- Host build output: shared `CARGO_TARGET_DIR=/home/peter/.cargo-target` only.
- PostgreSQL target: PG18 through the repository's normal pgrx test surface.
- Fixture/storage format: focused pgrx test databases only; no corpus, bench
  cluster, custom run directory, or retained PGDATA.

## Artifacts

- `cargo-pgrx-test-payload-projection-pg18.log`
  - Command: `cargo pgrx test pg18 test_distann_payload_projection_contract`
  - Purpose: Task 222 exact-mask/fallback control contract.
  - Result: `ok. 1 passed; 0 failed`; command exit code 0 at
    `2026-08-27 12:54:13-07:00`.
- `cargo-pgrx-test-sidecar-materialization-pg18.log`
  - Command: `cargo pgrx test pg18 test_distann_cover_sidecar_owner_materialization_fail_closed`
  - Purpose: local/remote selection, visibility, corruption, and fail-closed
    materialization behavior.
  - Result: `ok. 1 passed; 0 failed`; command exit code 0 at
    `2026-08-27 13:01:41-07:00`.
- `cargo-pgrx-test-sidecar-dml-pg18.log`
  - Command: direct exact invocation of the already-built shared-target pgrx
    test binary for `tests::pg_test_distann_cover_sidecar_dml_atomicity`, with
    the same `pg18 pg_test` no-default-feature environment used by
    `cargo pgrx test`.
  - Purpose: local owner insert/replacement/delete and rollback atomicity. The
    fixture calls `insert_from_owner_payload_for_test`, which skips only the
    production `record_physical_insert_intent` remote RPC; it proves atomicity
    across the graph, row-tier, and sidecar relations, not end-to-end intent
    recording through production's `remote_endpoint.rs` caller.
  - Result: `ok. 1 passed; 0 failed`; command exit code 0 at
    `2026-08-27 13:07:14-07:00`.

Each log records its command exit status. The focused checks used the shared
Cargo target and the repository's existing pgrx PG18 test surface; they did
not create a task-specific target, benchmark fixture, corpus, or run directory.
