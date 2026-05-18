# Review Request: SPIRE Production Session Consistency Policy

## Summary

Code checkpoint: `8befc3d72c2a58204dccdebed92e783c5f8ce0c3`

This slice starts the C5 AM-boundary strict/degraded policy by making the
production consistency mode a session-level enum rather than a per-call
free-form string:

- Added `ec_spire.remote_search_consistency_mode` with enum values `strict`
  and `degraded`, defaulting to `strict`.
- Added
  `ec_spire_remote_search_production_executor_session_summary(...)`, which
  reads that GUC and threads the resulting single mode into production executor
  state.
- The session summary reports `consistency_mode_source`, `consistency_mode`,
  dispatch count, degraded skip counters, and executor status.
- PG18 coverage proves the default strict mode is read from the GUC, degraded
  mode is read after `SET LOCAL`, and degraded mode still has to match the
  active epoch consistency policy before dispatch planning succeeds.

This does not yet wire the full AM scan path. It establishes the source of
truth C5 should consume unless a later reviewed packet replaces the GUC with an
explicit query option.

## Key Files

- `src/am/ec_spire/options.rs`
  - `ec_spire.remote_search_consistency_mode`
- `src/am/ec_spire/root/remote_candidates.rs`
  - production session summary reads the GUC before entering executor state
- `src/am/ec_spire/root/types.rs`
  - compact session summary row
- `src/lib.rs`
  - SQL-visible session summary
  - `test_ec_spire_prod_executor_session_policy_guc`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo pgrx test pg18 prod_executor_session_policy`
- `git diff --check -- src/am/ec_spire/options.rs src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`

## Review Questions

- Is a Userset enum GUC the right first AM-boundary source for strict/degraded
  mode, or should C5 insist on a query option instead?
- Is it correct that session degraded mode must still match the active epoch's
  published consistency mode before dispatch planning succeeds?
- Is the compact session summary enough proof for this slice, or should the
  full production state summary grow a `consistency_mode_source` column too?
