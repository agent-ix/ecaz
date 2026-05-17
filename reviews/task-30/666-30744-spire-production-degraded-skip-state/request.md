# Review Request: SPIRE Production Degraded Skip State

## Summary

Code checkpoint: `c2ef2bd7c04bf5baeb5e3948d4dd0fcdab76c8e0`

This slice starts Phase 11 C4 production strict/degraded semantics in executor
state:

- Added a `DegradedSkipped` production dispatch state.
- Degraded mode can skip transport failures, conninfo secret failures, and
  compact-candidate receive failures without turning them into strict
  transport/receive failures.
- Production summaries now expose `degraded_skipped_dispatch_count` and
  `first_degraded_skip_category`.
- Compact merge ignores degraded-skipped dispatches and consumes only ready
  candidate batches.
- Strict mode still uses the existing fail-closed paths.

This is not full C4. AM-boundary policy and the full fault matrix still need to
land before degraded mode is production-complete.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
  - `SpireRemoteProductionDispatchState::DegradedSkipped`
  - degraded-aware transport and candidate receive application helpers
  - merge behavior that ignores degraded-skipped dispatches
  - Rust state tests for degraded transport, missing secret, and receive failure
- `src/am/ec_spire/root/types.rs`
  - new production summary fields
- `src/lib.rs`
  - SQL-visible production state summary columns for degraded skip diagnostics
  - PG18 dry summary test coverage for the new columns
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test --no-default-features --features pg18 production_executor_ --lib`
- `cargo pgrx test pg18 production_executor_state_summary_is_dry`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/types.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`

## Review Questions

- Is `DegradedSkipped` the right explicit state for partial remote failures
  before AM scan integration?
- Should degraded skips stay out of `transport_failed_dispatch_count` and
  `candidate_receive_failed_dispatch_count`, with their own skip counter and
  first category?
- Is it acceptable that compact merge ignores degraded-skipped dispatches and
  errors only on non-ready, non-skipped states?

