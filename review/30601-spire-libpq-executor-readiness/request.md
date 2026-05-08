# SPIRE Libpq Executor Readiness

## Summary

This checkpoint adds a SQL-visible readiness surface for the remaining libpq
executor steps without opening sockets.

Changes:

- Adds `ec_spire_remote_search_libpq_executor_readiness(...)`.
- Splits the broad `requires_libpq_transport` gate into concrete executor
  steps:
  - conninfo secret resolution
  - libpq connection open
  - pipeline mode
  - remote search request send
  - receive validation
  - merge handoff
- Reports `ready`/`none` for local-only plans with no remote dispatch.
- Reports `remote_node_descriptor` as the next executor step for
  descriptor-blocked plans.
- Reports `requires_libpq_executor` and `conninfo_secret_resolution` once
  registered active/draining descriptors make dispatch pipeline-ready.
- Updates the Phase 7 task note with the executor-readiness surface.

## Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `9062f129`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_search_libpq_req`
- `cargo pgrx test pg18 remote_node_descriptor_catalog_active`
- `cargo fmt`
- Restored known unrelated rustfmt churn in:
  - `src/am/ec_ivf/scan.rs`
  - `src/am/ec_spire/options.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/update.rs`
- `git diff --check`

Result:

- PG18 `remote_search_libpq_req` filter passed:
  - `pg_test_ec_spire_remote_search_libpq_req_blocked`
  - `pg_test_ec_spire_remote_search_libpq_req_local`
- PG18 `remote_node_descriptor_catalog_active` filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`
- The tests cover descriptor-blocked, local-only ready/no-op, and
  active-descriptor executor-required states.

## Notes

This is still pre-I/O. It makes the future executor's first unresolved work item
explicit instead of hiding every remote-ready dispatch behind the generic
transport gate.
