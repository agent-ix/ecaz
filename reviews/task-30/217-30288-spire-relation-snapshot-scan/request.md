# SPIRE Relation Snapshot Scan

## Checkpoint

- Code commit: `28fcc084` (`Load SPIRE relation snapshot for scans`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: relation-backed active-epoch `amrescan` loading

## Summary

This checkpoint makes populated relation-backed SPIRE indexes readable by the initial quantized scan path:

- `amrescan` now reads active epoch/object/placement manifest tuples from root/control locators.
- The decoded manifests are validated as a published epoch snapshot.
- The scan path uses `SpireRelationObjectStore` to read the root routing object and segmented V2 leaf objects.
- Existing helper-level routing, `nprobe`, quantized candidate scoring, candidate limiting, and cursor output are reused for live scans.
- The populated-build PG18 test now also forces an ordered index scan and verifies the nearest row is returned.

Exact heap rerank is still not wired; the current live path preserves the quantized candidate score in the rerank callback slot. PQ-FastScan scoring also remains blocked until persisted grouped-PQ model metadata exists.

## Changed Files

- `src/am/ec_spire/scan.rs`
- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo fmt`
  - Completed; existing stable rustfmt warnings for unstable `imports_granularity` / `group_imports`.
- `cargo test --lib test_ec_spire_populated_build_publishes_root_control --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1076 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `196 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`
- `git diff --check`
  - clean
- `git diff --cached --check`
  - clean

## Notes

- This is not a recall/latency checkpoint.
- Insert-after-build, delete/vacuum cleanup, exact heap rerank, and relation-backed admin diagnostics remain open.
- No measurement artifacts are included; validation is functional PG18 coverage only.
