# Review Request: Task 28 IVF Query Prep

Scope: Phase 4 query-prep checkpoint. `ec_ivf` rescans now validate the
ORDER BY query, cache query/prepared state, score persisted centroids, and
store the selected probe-list IDs for the future posting-list scan path.

Task: `plan/tasks/28-ivf-access-method.md` Phase 4

Branch: `task28-ivf`

Head SHA: `b89c2f7d141a254ccb774f990973f5d13d0538a0`

Owner: coder2

Files:

- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/mod.rs`
- `src/am/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy.
- The new PG tests were compiled but not run. No test suite was executed for
  this checkpoint.
- No measurement claim is made in this packet.

## Summary

This slice implements the first scan-time state for IVF:

- `amrescan` now rejects index quals, missing/multiple ORDER BY keys, NULL
  query datums, empty queries, oversized queries, and dimension mismatches.
- Scan opaque state now stores the raw query, metadata dimensions/list counts,
  effective `nprobe`, a prepared default quantizer scorer, centroid scores, and
  selected list IDs.
- Empty indexes keep query state but intentionally allocate no prepared scorer,
  centroid scores, or selected lists.
- Adds physical centroid readback from `centroid_head`, mirroring the directory
  readback helper added in the prior slice.
- Adds PG debug coverage for empty query prep and non-empty `nprobe` selection.

## Review Focus

Please review for:

- Whether `nprobe = 0` should resolve to `ceil(sqrt(nlists))` for this first
  implementation, or whether auto should mean full-probe until recall gates
  exist.
- Whether using the default canonical quantizer scorer for Phase 4 prep is the
  right temporary contract before storage-format-specific scoring is wired.
- Whether centroid reads can rely on the same physical-contiguous tuple walk as
  directory reads, or whether centroids should get explicit refs/links sooner.
- Whether the scan opaque memory ownership is correct across repeated rescans
  and `amendscan`.

## Non-Goals

This packet does not read posting-list candidates, emit tuples, set order-by
scores, implement rerank mode, live insert, vacuum, planner costing, or any
measurement claim.
