# Review Request: Task 41 unsafe surface strategy

Code commit: `7df79b8cf70a439e3ad550ff68ab74a14361462f`

## Summary

This packet pivots Task 41 from opportunistic relation-guard slices to a
survey-driven burndown strategy.

- Pinned the current unsafe-comment baseline at 4,579 entries.
- Grouped the remaining surface by safety mechanism rather than by raw count.
- Identified which clusters should be deleted by wrappers, which belong to
  Task 40 or Task 43, and which should wait for Task 35 residual annotation.
- Updated Task 41 tracking so the next coder pickup uses this survey packet
  before cutting additional wrapper slices.

## Baseline

- Current baseline: 4,579 entries.
- Files represented: 106.
- Largest directories: `src/am` with 3,606 entries, `src/tests` with 539,
  `src` with 366, and `src/quant` with 68.

## Strategy

1. Finish the current PG relation-resource track.
   - Migrate remaining `open_valid_ec_*_index` callers to
     `AccessShareIndexRelation`.
   - Split validation-only callers from AM callers that need a live relation.
   - Delete the raw compatibility helpers once their callers are gone.
2. Open one PG buffer/WAL resource wrapper track.
   - Start with page-codec and WAL-heavy files such as `src/am/ec_ivf/page.rs`,
     `src/am/ec_spire/page.rs`, and `src/storage/wal.rs`.
   - Target paired resources first: buffer pin/release, lock/unlock, generic
     WAL start/finish/abort.
3. Defer synchronization primitives to Task 40.
   - `src/am/ec_hnsw/shared.rs`, `src/am/common/parallel.rs`, and parallel
     build state should not be annotated piecemeal before the lift.
4. Use Task 43 before claiming proof on pointer-heavy residuals.
   - Page raw pointer arithmetic, scan descriptor manipulation, tuple slots,
     detoast lifetimes, and SIMD/quantization should get Miri or focused proof
     coverage before Task 35 comments are written.
5. Keep test-only unsafe separate from production burndown.
   - Test debug exports can migrate after production wrappers, unless they
     block deletion of a raw helper.

## Reviewer Focus

- Confirm the grouping puts structural deletion ahead of comment backfill.
- Confirm the Task 35 deferral rule is explicit enough to avoid racing Task 41
  and Task 40.
- Confirm the next implementation slices should be selected from the relation
  helper and PG resource wrapper clusters, not from the largest raw-count files
  alone.

## Artifacts

- `artifacts/baseline-report.log`
- `artifacts/strategy-matrix.md`
- `artifacts/manifest.md`
