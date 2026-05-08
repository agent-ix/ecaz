# SPIRE Libpq Connection Plan

## Summary

This checkpoint adds the next coordinator transport planning surface without
opening libpq connections.

Changes:

- Adds `ec_spire_remote_search_libpq_connection_plan(...)` and
  `ec_spire_remote_search_libpq_connection_summary(...)`.
- Adds `ec_spire_remote_search_libpq_dispatch_plan(...)` and
  `ec_spire_remote_search_libpq_dispatch_summary(...)`.
- Resolves remote request rows against `ec_spire_remote_node_descriptor`.
- Exposes per-node secret reference, remote index regclass, remote identity byte
  count, pipeline mode, and transport status.
- Keeps raw conninfo out of SQL-visible state; only `conninfo_secret_name` is
  surfaced.
- Preserves fail-closed behavior for missing descriptors by reporting
  `requires_remote_node_descriptor` and no pipeline mode.
- Aggregates descriptor-resolved, missing-descriptor, pipeline, remote-PID, and
  blocked-PID counts into one coordinator gate row.
- Exposes the pre-I/O dispatch action, receive validator, request shape, and
  fail-closed dispatch counts for the future libpq pipeline executor.
- Updates the Phase 7 task note with the connection and dispatch envelope
  surfaces.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `68fac3e0`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_search_libpq_req`
- `cargo pgrx test pg18 remote_node_descriptor_catalog_active`
- `git diff --check`

Result:

- PG18 `remote_search_libpq_req` filter passed:
  - `pg_test_ec_spire_remote_search_libpq_req_blocked`
  - `pg_test_ec_spire_remote_search_libpq_req_local`
- PG18 `remote_node_descriptor_catalog_active` filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`
  - Confirms the connection summary reports one descriptor-resolved pipeline
    connection and preserves `requires_libpq_transport`.
  - Confirms the dispatch plan reports
    `open_pipeline_and_send_remote_search` plus
    `validate_remote_search_candidate_batch` for registered descriptors.

## Notes

This is still pre-execution transport groundwork. The new plans prove the
future executor can consume descriptor-backed connection metadata, pipeline mode
requirements, request shape, and receive validation expectations, but they do
not resolve secret values or open libpq connections.
