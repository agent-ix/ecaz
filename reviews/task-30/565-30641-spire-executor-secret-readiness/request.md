# SPIRE Executor Secret Readiness

## Scope

Task 30 SPIRE Phase 7 now feeds the remote search conninfo-secret summary into
the executor-readiness and coordinator gate surfaces. A resolved secret no
longer leaves the reported next blocker at secret resolution; it advances to
the next pre-I/O executor step.

Code checkpoint: `3d40f604` (`Advance SPIRE executor readiness after secret resolution`)

## Changes

- Reworked `remote_search_libpq_executor_readiness_row(...)` to derive dispatch
  rows once, summarize dispatch readiness, derive the conninfo-secret plan from
  those same rows, and summarize secret readiness.
- Replaced the dispatch-only executor-readiness helper with a summary-based
  helper that preserves descriptor blockers, preserves the executor step-action
  contract, and advances resolved secret plans to `open_libpq_connection`.
- Updated the coordinator pipeline to reuse the same secret summary when
  calculating `libpq_executor_next_step` and `next_blocker`.
- Extended the active remote-node catalog PG18 fixture so scoped secret
  resolution now proves coordinator and executor readiness advance from
  `conninfo_secret_resolution` to `open_libpq_connection`.
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

- Whether `secret_resolution_action` should continue to expose the stable step
  contract action after resolution, or whether a future surface should add an
  explicit completed-step status column.
- Whether the next libpq slice should add a connection-open plan surface before
  adding any actual socket I/O.
