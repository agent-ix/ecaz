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
- Follow-up for reviewer feedback: connection descriptor resolution now loads
  only `active` and `draining` catalog rows, so `failed` or `disabled`
  descriptors cannot report `secret_reference_ready` or `libpq_pipeline`.
- Aggregates descriptor-resolved, missing-descriptor, pipeline, remote-PID, and
  blocked-PID counts into one coordinator gate row.
- Exposes the pre-I/O dispatch action, receive validator, request shape, and
  fail-closed dispatch counts for the future libpq pipeline executor.
- Threads libpq dispatch count/status into
  `ec_spire_remote_search_coordinator_gate_summary(...)`.
- Updates the Phase 7 task note with the connection and dispatch envelope
  surfaces.

## Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

Head SHA: `9866d033`

- `cargo check --lib --no-default-features --features pg18`
- `cargo pgrx test pg18 remote_search_libpq_req`
- `cargo pgrx test pg18 remote_search_coordinator_gate_summary`
- `cargo pgrx test pg18 remote_node_descriptor`
- `cargo pgrx test pg18 remote_node_desc_failed_blocks_libpq_dispatch`
- `git diff --check`

Result:

- PG18 `remote_search_libpq_req` filter passed:
  - `pg_test_ec_spire_remote_search_libpq_req_blocked`
  - `pg_test_ec_spire_remote_search_libpq_req_local`
- PG18 `remote_search_coordinator_gate_summary` filter passed:
  - `pg_test_ec_spire_remote_search_coordinator_gate_summary`
  - Confirms the top-level coordinator gate reports libpq dispatch count/status
    for local-only and missing-descriptor plans.
- PG18 `remote_node_descriptor` filter passed:
  - `pg_test_ec_spire_remote_node_descriptor_registration_contract`
  - `pg_test_ec_spire_remote_node_descriptor_contract`
  - `pg_test_ec_spire_remote_node_descriptor_readiness_missing`
  - `pg_test_ec_spire_remote_node_descriptor_catalog_active`
  - `pg_test_ec_spire_remote_node_descriptor_stale_generation_rejected`
  - Confirms the connection summary reports one descriptor-resolved pipeline
    connection and preserves `requires_libpq_transport`.
  - Confirms the dispatch plan reports
    `open_pipeline_and_send_remote_search` plus
    `validate_remote_search_candidate_batch` for registered descriptors.
- PG18 `remote_node_desc_failed_blocks_libpq_dispatch` filter passed:
  - `pg_test_ec_spire_remote_node_desc_failed_blocks_libpq_dispatch`
  - Confirms the connection plan reports `requires_remote_node_descriptor`,
    `pipeline_mode = none`, and blocked dispatch for a registered `failed`
    descriptor.

## Notes

This is still pre-execution transport groundwork. The new plans prove the
future executor can consume descriptor-backed connection metadata, pipeline mode
requirements, request shape, and receive validation expectations, but they do
not resolve secret values or open libpq connections.
