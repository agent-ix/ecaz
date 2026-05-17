# Review Request: SPIRE Production Strict/Degraded Fault Matrix

## Summary

Code checkpoint: `0cbf841cb2f5e678d1bb7df4568cdc7ca8010eae`

This slice closes the Phase 11.4 C4 matrix planning gap before C5 AM
integration:

- Added `ec_spire_remote_search_production_fault_matrix()`, a dry SQL-visible
  production policy table for strict/degraded failure behavior.
- The matrix covers connect/auth/cert-style transport failures, conninfo secret
  failures, remote and local statement timeouts, backend termination, remote and
  local query cancellation, candidate validation/decode, endpoint identity,
  protocol and extension version skew, stale/served epoch, requested epoch,
  `consistency_mode_mismatch`, missing remote index, and reserved Stage D remote
  heap-resolution categories.
- The matrix makes local cancellation query-wide in both strict and degraded
  modes, while most remote-node failures fail closed in strict mode and skip the
  affected node in degraded mode.
- Phase 11 and the coordinator/executor design now point to this surface as the
  C4 policy contract that C5 should consume.

This does not implement Stage D heap resolution. The heap rows are reserved
policy categories so the AM boundary has stable names before final heap
resolution starts returning SQL rows.

## Key Files

- `src/am/ec_spire/root/remote_candidates.rs`
- `src/am/ec_spire/root/types.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/task30-phase11-spire-distributed-production-parity.md`
- `plan/design/spire-production-coordinator-executor.md`

## Validation

Packet-local logs are in `artifacts/` and indexed in
`artifacts/manifest.md`.

- `cargo fmt --check`
- `cargo check --no-default-features --features pg18`
- `cargo test production_fault_matrix_covers_required_categories --no-default-features --features pg18`
- `cargo pgrx test pg18 production_fault_matrix_contract`
- `git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/am/ec_spire/root/types.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md`

## Review Questions

- Is the row shape sufficient for C5 to consume: category, scope, executor step,
  strict action/status, degraded action/status, and recommendation?
- Should reserved Stage D heap categories stay in this C4 matrix now, or wait
  until the heap executor emits them?
- Are `consistency_mode_mismatch` and `requested_epoch_mismatch` correctly
  fail-closed in degraded mode rather than treated as degraded remote skips?
