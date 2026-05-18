# SPIRE Scan Path Complete

## Checkpoint

- Code commit: `2ab83a33`
  (`Mark SPIRE scan path complete`)
- Branch: `task30-spire-partition-object-spec`
- Task: Task 30 SPIRE IVF foundation
- Scope: Task-plan closeout for Phase 1 local single-store scan path

## Summary

This checkpoint marks the Phase 1 scan path complete for supported SPIRE
payload formats.

The already-landed scan path now covers:

- root/control and active epoch manifest loading during `amrescan`
- root routing to top-`nprobe` leaf PIDs
- strict/degraded placement handling in helper paths
- V2 leaf scans with batched TurboQuant/RaBitQ scoring
- heap rerank for `ecvector` and `tqvector`
- row-encoded insert-delta inclusion
- delete-delta suppression for base and delta-insert candidates
- deterministic bounded candidate ordering and cursor output
- empty active epochs, including empty `pq_fastscan` indexes, returning no rows

Populated SPIRE PQ-FastScan remains build-blocked until grouped-PQ
model/codebook metadata is persisted, so that work is tracked as future
storage/scorer binding rather than a Phase 1 scan-path blocker.

## Changed Files

- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `git diff --check`
- `git diff --cached --check` before commit

Tests were not rerun for this documentation-only closeout. The immediately
preceding checkpoint (`30329`) ran:

- `cargo test --lib test_ec_spire_empty_pq_fastscan_build_scan_no_rows --no-default-features --features pg18 -- --nocapture`
  - `1 passed; 0 failed; 0 ignored; 0 measured; 1115 filtered out`
- `cargo test --lib ec_spire --no-default-features --features pg18`
  - `235 passed; 0 failed; 0 ignored; 0 measured; 881 filtered out`

## Notes

- This does not close measured recall/latency evidence, physical old-epoch
  cleanup, SQL `VACUUM` end-to-end coverage, insert batching, concurrency
  stress, or SPIRE PQ-FastScan model persistence.
