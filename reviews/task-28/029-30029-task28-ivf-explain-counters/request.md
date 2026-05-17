# Review Request: Task 28 IVF EXPLAIN Counters

Status: open
Owner: coder2
Code checkpoint: 891da15b238065dfd5d38f5ba74045f46b488d19
Branch: task28-ivf
Date: 2026-04-25

## Scope

This packet covers the Phase 7 EXPLAIN-counter checkpoint for the first
`ec_ivf` access method baseline.

Changes in scope:

- Add IVF-specific EXPLAIN counters for centroid scores, selected lists,
  posting pages read, candidates scored, rerank rows, and filtered duplicates.
- Store and reset those counters in `ec_ivf` scan opaque state.
- Extend the shared PG18 `EXPLAIN (ecaz)` hook to dispatch to `ec_hnsw` or
  `ec_ivf` counters based on the index access method.
- Add PG18 coverage that verifies `EXPLAIN (FORMAT JSON, ecaz, ANALYZE)`
  emits `Ecaz Stats` with IVF counter names.
- Keep the existing HNSW EXPLAIN path covered after the dispatch change.
- Mark the Phase 7 EXPLAIN-counter checklist item complete in
  `plan/tasks/28-ivf-access-method.md`.

## Files

- `src/am/common/explain.rs`
- `src/am/ec_ivf/scan.rs`
- `src/am/ec_ivf/mod.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

## Validation

PG18-focused validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_pg18_explain_option_emits_ecaz_stats_group_for_ec_ivf`
- `cargo pgrx test pg18 test_pg18_explain_option_emits_ecaz_stats_group`
- `cargo test --no-default-features --features pg18 ivf_explain --lib`
- `git diff --check`

PostgreSQL version: 18.3 via pgrx `pg18`.

No measurement claims are made in this packet.

## Review Focus

- Whether the IVF counter names are the right operator-facing vocabulary for
  the first baseline.
- Whether the shared EXPLAIN hook should dispatch by AM name as implemented, or
  if this should move behind an AM-local trait/helper shape before more AMs
  arrive.
- Whether `posting_pages_read` should continue counting block ranges from list
  directory head/tail refs, or be tightened later to count lower-level page
  reads directly.

## Non-Goals

- Shared pgstat aggregation for IVF counters.
- ReadStream instrumentation.
- Rerank implementation; `rerank_rows` remains zero while rerank is disabled.
- Measurement artifacts.
