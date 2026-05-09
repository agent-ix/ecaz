# SPIRE Remote Search Connection-Open Plan

## Scope

Task 30 SPIRE Phase 7 now has the next pre-I/O executor surface after
conninfo-secret resolution: per-node connection-open work for remote search.

Code checkpoint: `9258949b` (`Add SPIRE remote search connection-open plan`)

## Changes

- Added `ec_spire_remote_search_libpq_connection_open_plan(...)`.
- Added `ec_spire_remote_search_libpq_connection_open_summary(...)`.
- The plan derives from the secret plan and advances resolved secret rows to
  `open_libpq_connection` / `enter_libpq_pipeline_mode` while preserving
  descriptor or secret blockers before connection work.
- The rows publish per-query/no-pooling lifecycle policy and the resolved
  conninfo byte count, but still do not expose raw conninfo or open sockets.
- Extended the active remote-node catalog PG18 fixture to assert the connection
  plan and summary advance to the pipeline-mode blocker after a scoped secret
  resolves.
- Updated the Phase 7 task note.

## Validation

- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `cargo pgrx test pg18 test_ec_spire_remote_node_descriptor_catalog_active`
- `git diff --check`

## Review Focus

- Whether the next slice should add a pipeline-mode/send plan surface or start
  the actual libpq socket executor behind the existing plan rows.
