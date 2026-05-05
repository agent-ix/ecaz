# Review Request: SPIRE Flat vs Recursive SQL Comparison

Head SHA: `38f2bf7e`

## Summary

The Phase 3 comparison proof now reaches relation-backed SQL. A new PG18 smoke
builds a default flat `ec_spire` index and an opt-in recursive
`recursive_fanout = 2` `ec_spire` index over the same four-row dataset.

The test verifies the two indexes expose different hierarchy shapes while
returning the same nearest row for the same ordered query:

- flat: no internal routing objects, depth 1, unsupported recursive routing,
  and four root-routing rows;
- recursive: two internal routing objects, depth 2, supported recursive
  routing, and two root-routing rows;
- both ordered scans return row id `1` for query `[1.0, 0.0]`.

## Files

- `src/lib.rs`
- `plan/tasks/30-spire-ivf-foundation.md`

## Validation

- `cargo test flat_recursive_same_candidate -- --nocapture`
  - 1 passed, including PG18 pg-test
    `pg_test_ec_spire_flat_recursive_same_candidate`.
- `git diff --check`

## Review Focus

- Confirm the SQL comparison is sufficient for the Phase 3 final review-packet
  checklist item.
- Confirm the flat and recursive diagnostic expectations capture the intended
  user-visible distinction.
- Confirm comparing nearest-row equality is the right narrow smoke before
  broader recall/latency measurements.
