# Artifact Manifest

## Metadata

- Head SHA: `f985259d64c41e2b9565094ec45fe60af8b75d34`
- Packet/topic: `616-c1-pg18-rank-order-diagnostic`
- Timestamp: 2026-04-25
- Fixture: PG18 planner-visible parallel scan validation, 50,000 rows, 16 dimensions, randomized embeddings, `LIMIT 100`, `ef_search=500`
- Surface: isolated one-index-per-table fixture created by `ecaz-cli dev test pg18-parallel-scan`
- Storage format: default `ec_hnsw` scalar storage
- Rerank mode: default

## `pg18-parallel-50k-dim16-rank-order-default.log`

- Lane: default planner-visible parallel scan
- Command:
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-rank-order-default.log`
- Key lines:
  - `[pg18-parallel] env=[]`
  - `Limit (actual time=14.872..16.091 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
  - `Elements Skipped: 0`
  - `Heap TIDs Returned: 100`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Ordered After Visible Drops: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-50k-dim16-rank-order-contributor.log`

- Lane: existing contributor diagnostic
- Command:
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-rank-order-contributor.log`
- Key lines:
  - `[pg18-parallel] env=["TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC"]`
  - `Limit (actual time=40.486..41.847 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
  - `Elements Skipped: 0`
  - `Heap TIDs Returned: 100`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 260`
  - `Parallel Contributor Publish: Duplicate Active: 8`
  - `Parallel Contributor Publish: Handoff Ready: 0`
  - `Parallel Contributor Publish: Ordered After Visible: 252`
  - `Parallel Contributor Ordered After Visible Drops: 248`
  - `Parallel Contributor Duplicate Retires: 4`
  - `Parallel Contributor Output Limit Exits: 4`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-50k-dim16-rank-order-diagnostic.log`

- Lane: contributor diagnostic plus rank-order diagnostic
- Command:
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --env TQVECTOR_PG18_PARALLEL_RANK_ORDER_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-rank-order-diagnostic.log`
- Key lines:
  - `[pg18-parallel] env=["TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC", "TQVECTOR_PG18_PARALLEL_RANK_ORDER_DIAGNOSTIC"]`
  - `Limit (actual time=40.699..42.002 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 100`
  - `Elements Skipped: 1`
  - `Heap TIDs Returned: 100`
  - `Parallel Handoffs: Foreign Selected: 1`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 260`
  - `Parallel Contributor Publish: Duplicate Active: 8`
  - `Parallel Contributor Publish: Handoff Ready: 0`
  - `Parallel Contributor Publish: Ordered After Visible: 252`
  - `Parallel Contributor Ordered After Visible Drops: 248`
  - `Parallel Contributor Duplicate Retires: 4`
  - `Parallel Contributor Output Limit Exits: 4`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] validation failed`
  - Serial/candidate order diverges with `35325` before `1777` in serial output and `1777` before `35325` in candidate output.
