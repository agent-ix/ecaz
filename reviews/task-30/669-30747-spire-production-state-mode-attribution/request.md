# Review Request: SPIRE Production State Mode Attribution

## Summary

Code checkpoint: `81eaec07572c5fa8348d5106fdfa9de90237c6a6`

This follow-up addresses the safe part of reviewer feedback on `30746`:

- `SpireRemoteProductionExecutorStateSummaryRow` now carries
  `consistency_mode_source` and `consistency_mode`.
- Production executor state helpers thread the canonical mode label into Rust
  executor state, including degraded helper paths.
- Existing SQL full-state summary remains unchanged because its returned row is
  already at the pgrx tuple-width ceiling; SQL callers still get mode
  attribution through
  `ec_spire_remote_search_production_executor_session_summary(...)`.
- The Phase 11 task/design docs now record that distinction explicitly.

I intentionally did not change active-epoch mismatch error semantics in this
slice. That path currently raises a dispatch-planning error, so turning it into
a named `failure_category` should be handled with a separate surface design.

## Key Files

- `src/am/ec_spire/root/types.rs`
- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/tests.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test --no-default-features --features pg18 production_executor_ --lib`
- `cargo pgrx test pg18 prod_executor_session_policy`
- `git diff --check -- src/am/ec_spire/root/types.rs src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/tests.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`

## Review Questions

- Is Rust-side mode attribution in `SpireRemoteProductionExecutorStateSummaryRow`
  enough for C5, with SQL attribution kept on the compact session summary due
  to the full SQL row-width limit?
- Should the active-epoch policy mismatch taxonomy be a new row-returning
  preflight surface rather than changing the existing planner functions from
  fail-fast errors?
