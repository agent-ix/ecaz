# Review Request: Task 41 SPIRE DML Frontdoor Heap Guard

Code commit: `50590d445d35452e28fcbeace1bb754fb039b487`

## Summary

This checkpoint wraps the heap relation opened by
`dml_frontdoor_relation_context_catalog_row` in
`src/am/ec_spire/dml_frontdoor/mod.rs`.

- Adds `AccessShareHeapRelation` next to the existing index guard.
- Borrows the raw heap relation pointer only while the guard is live.
- Removes the manual `table_open` / `table_close` pair around catalog context
  loading and cache population.

## Safety Delta

- Baseline entries: `4313` -> `4311`.
- `src/am/ec_spire/dml_frontdoor/mod.rs`: `160` -> `158`.
- The remaining DML frontdoor residuals are planner/query-tree and SPI/catalog
  pointer access, not the catalog heap relation close path.

## Reviewer Focus

- Confirm the heap relation guard lifetime covers
  `dml_frontdoor_relation_context_catalog_for_open_heap`.
- Confirm cached `SpireDmlFrontdoorRelationContext` data is owned and does not
  borrow from the heap relation after the guard drops.
- Confirm the null-open error behavior is unchanged from the previous explicit
  null check.

## Validation

- `bash scripts/check_unsafe_comments.sh`
- `make fmt-check`
- `git diff --check`
- `bash scripts/unsafe_baseline_report.sh`
- `cargo check --all-targets --no-default-features --features pg18,bench`

Packet-local logs and baseline snapshots are in `artifacts/`; see
`artifacts/manifest.md`.
