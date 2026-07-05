# SPIRE Scan Sanity Diagnostics

## Checkpoint

- Code commit: `462d3368`
  (`Expose SPIRE scan sanity diagnostics`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: deterministic SQL diagnostics for scan recall-sanity preconditions

## Summary

This checkpoint adds a lightweight scan sanity surface before any measured
recall/latency packet:

- Added `ec_spire_index_scan_sanity_snapshot(index_oid)` as a stable, strict
  SQL table function.
- The function reports active epoch, active leaf count, effective `nprobe`,
  effective `rerank_width`, and source labels for both resolved scan knobs.
- It labels whether the resolved scan covers every active leaf and whether
  `rerank_width = 0` requests full-frontier rerank.
- It returns conservative `recall_sanity_status`, `latency_risk_status`, and
  recommendation text so operators can tell approximate bounded-leaf scans
  from exact-leaf/full-frontier sanity checks.
- Updated the Task 30 plan to distinguish deterministic recall-sanity
  precondition reporting from measured recall/latency evidence.

This is not a recall or latency measurement. It does not claim quality,
throughput, or p95/p99 behavior; it only reports whether the current scan
configuration has the preconditions normally used for an exact coverage sanity
check.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_scan_sanity_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1096 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `216 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean before code commit

## Notes

- No measurement artifacts are included because this packet does not make a
  measurement claim.
- Exact-leaf/full-frontier status is a configuration diagnostic, not an
  assertion that recall has been measured.
- This does not implement boundary replication, PQ-FastScan scorer binding,
  physical object cleanup, real SQL VACUUM end-to-end behavior, or measured
  recall/latency evidence.
