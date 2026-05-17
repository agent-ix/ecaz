# Review Request: Task 28 IVF Probe Candidates

Scope: Phase 4 posting-list candidate checkpoint. `ec_ivf` rescans now read
the selected posting-list block ranges, decode postings, score candidates, and
store score-ordered candidates for the upcoming tuple-emission slice.

Task: `plan/tasks/28-ivf-access-method.md` Phase 4

Branch: `task28-ivf`

Head SHA: `99e73899db88021b180186ad16e9d4538b486ea4`

Owner: coder2

Files:

- `src/am/ec_ivf/page.rs`
- `src/am/ec_ivf/scan.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy.
- The updated PG tests were compiled but not run. No test suite was executed
  for this checkpoint.
- No measurement claim is made in this packet.

## Summary

This slice adds the first persisted posting-list read path:

- Page helpers can now scan a list's head/tail block range, skip non-posting
  tuples, decode posting tuples, and filter by list ID.
- `amrescan` now materializes candidates from selected probe lists after
  centroid routing.
- Candidate scoring uses the prepared default quantizer query and stores the
  PostgreSQL ORDER BY score shape as negative inner product.
- Duplicate heap TIDs are suppressed across all selected lists before
  candidates are sorted by score and heap TID.
- PG debug coverage now verifies that empty indexes materialize zero
  candidates and a three-list, two-probe fixture materializes two candidates.

## Review Focus

Please review for:

- Whether block-range list scans plus list-ID filtering are acceptable until
  directory entries grow explicit tuple-level list bounds.
- Whether candidate materialization should stay all-candidates-sorted for the
  next result-emission slice, or move immediately to bounded top-k state.
- Whether default canonical quantizer scoring is acceptable until
  storage-format-specific scoring and rerank mode are wired.
- Whether the palloc-owned candidate array is safe across repeated rescans and
  `amendscan`.

## Non-Goals

This packet does not emit tuples, set order-by score slots, implement bounded
top-k state, rerank mode, live insert, vacuum, planner costing, or any
measurement claim.
