# Task 39 SPIRE storage coverage raise

## Summary

Snapshots fresh `make coverage` data and lifts the
`fixtures/quality/coverage-baseline.tsv` numbers for the four SPIRE
storage files touched by packets 028–031 in this branch:

| File | Old baseline | New (this packet) | Delta |
| --- | --- | --- | --- |
| `am/ec_spire/storage/leaf_v2_parts.rs` | 77.52% | **81.03%** | +3.51 (crosses 80% target) |
| `am/ec_spire/storage/local_store.rs`   | 78.21% | **80.07%** | +1.86 (crosses 80% target) |
| `am/ec_spire/storage/local_store_set.rs` | 41.52% | **63.74%** | +22.22 |
| `am/ec_spire/storage/vec_id.rs`         | 69.05% | **69.64%** | +0.59 |

`leaf_v2.rs` did not move in this session — its 71.76% baseline is
preserved as-is (no targeted test was added for that file).

This is the explicit ratchet packet for the 028–031 test-coverage
slices on the same branch: the test commits land the coverage, this
packet records the new baseline.

## Code under review

- Commit: pending (this packet only edits
  `fixtures/quality/coverage-baseline.tsv`).
- Changed file: `fixtures/quality/coverage-baseline.tsv`.

## Validation

- `make coverage` — 455 careful-crate tests passed, summary captured
  at `artifacts/coverage/summary.txt` (+ `careful-summary.txt`,
  `root-summary.txt`, raw JSON files).
- `scripts/check_coverage_delta.sh` with the 4-path `changed-files.txt`
  — `coverage ok` for every checked path. Artifact:
  `artifacts/coverage-delta-check.log`.
- `scripts/check_coverage_baseline_complete.sh` — `coverage baseline
  complete for 40 critical paths`. Artifact:
  `artifacts/coverage-baseline-check.log`.

## Notes

- Source slices that drove these numbers:
  - 028 `vec-id-coverage-tighten`: global-max + `local_sequence()`
    None on `SpireVecId`.
  - 029 `leaf-v2-meta-validate`: 9 `validate()` error branches in
    `SpireLeafPartitionObjectV2Meta`.
  - 030 `local-store-edge-cases`: `new(_, 0)` + top-graph epoch 0.
  - 031 `local-store-set-non-leaf`: routing / delta / top-graph
    insert+read delegations.
- The two below-target files that remain in this group are
  `vec_id.rs` (69.64%) and `local_store_set.rs` (63.74%);
  `leaf_v2.rs` is also still below at 71.76%. Each is a candidate for
  a future Task 39 coverage slice — both `vec_id.rs` and
  `leaf_v2_parts.rs` have private helpers (`SpireVecIdKind::decode`
  error path; `SpireLeafPartitionObjectV2Segment::validate_against_meta`
  branches) that need a different test seam.
