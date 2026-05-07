# SPIRE Effective Options SQL

## Checkpoint

- Code commit: `27722e91` (`Expose SPIRE effective scan options`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: active leaf count and effective scan option resolution in
  `ec_spire_index_options_snapshot`

## Summary

This checkpoint extends the SPIRE options SQL surface from raw configuration
echoing to active scan-option resolution:

- `ec_spire_index_options_snapshot(index_oid)` now loads the active SPIRE
  root/control state and, when an active epoch exists, counts root routing
  children to report `active_leaf_count`.
- The snapshot now reports `effective_nprobe` plus
  `effective_nprobe_source`, using the same session/relation/auto resolution
  as live scans and clamping session requests to the active leaf count.
- The snapshot now reports `effective_rerank_width` plus
  `effective_rerank_width_source`, using the same session/relation resolution
  as live scans.
- `session_nprobe` now follows the scan resolver semantics: only session values
  of 1 or higher are treated as an override.
- The focused PG18 SQL test now builds a populated three-leaf `ec_spire` index,
  sets session overrides, and verifies active leaf count plus effective scan
  option resolution.
- The Task 30 plan now records effective scan-option diagnostics as covered.

This does not add recall/latency evidence, physical cleanup, real SQL `VACUUM`
end-to-end coverage, or PQ-FastScan scorer binding.

## Changed Files

- `src/am/ec_spire/mod.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable
    `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_options_snapshot_sql --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1083 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `203 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- No measurement artifacts are included; validation is functional PG18 coverage
  only.
- Real SQL `VACUUM` end-to-end validation remains open; psql access to the
  local test sockets is blocked in the current sandbox without escalation.
