# Artifact Manifest

## Metadata

- Head SHA: `6c8fd74badab57bcbde0cedd74118228f0d92c67`
- Packet/topic: `615-c1-pg18-contributor-ordered-wait`
- Timestamp: 2026-04-25
- Fixture: PG18 planner-visible parallel scan validation, 50,000 rows, 16 dimensions, randomized embeddings, `LIMIT 100`, `ef_search=500`
- Surface: isolated one-index-per-table fixture created by `ecaz-cli dev test pg18-parallel-scan`
- Storage format: default `ec_hnsw` scalar storage
- Rerank mode: default

## `pg18-parallel-50k-dim16-ordered-wait-default.log`

- Lane: default planner-visible parallel scan
- Command:
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-ordered-wait-default.log`
- Key lines:
  - `[pg18-parallel] env=[]`
  - `Limit (actual time=13.819..15.071 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
  - `Heap TIDs Returned: 100`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Publish: Duplicate Active: 0`
  - `Parallel Contributor Publish: Handoff Ready: 0`
  - `Parallel Contributor Publish: Ordered After Visible: 0`
  - `Parallel Contributor Ordered After Visible Drops: 0`
  - `Parallel Contributor Output Limit Exits: 0`
  - `Parallel Contributor Poll Limit Exits: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-50k-dim16-ordered-wait-contributor.log`

- Lane: existing contributor diagnostic
- Command:
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-ordered-wait-contributor.log`
- Key lines:
  - `[pg18-parallel] env=["TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC"]`
  - `Limit (actual time=41.039..42.706 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
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
  - `Parallel Contributor Poll Limit Exits: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-50k-dim16-ordered-wait-diagnostic.log`

- Lane: contributor diagnostic plus ordered-wait diagnostic
- Command:
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_ORDERED_WAIT=1 --log-output target/pg18-parallel-50k-dim16-ordered-wait-diagnostic.log`
- Key lines:
  - `[pg18-parallel] env=["TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC", "TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_ORDERED_WAIT"]`
  - `Limit (actual time=33.703..35.081 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
  - `Heap TIDs Returned: 100`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 12`
  - `Parallel Contributor Publish: Duplicate Active: 8`
  - `Parallel Contributor Publish: Handoff Ready: 0`
  - `Parallel Contributor Publish: Ordered After Visible: 4`
  - `Parallel Contributor Ordered After Visible Drops: 0`
  - `Parallel Contributor Duplicate Retires: 4`
  - `Parallel Contributor Output Limit Exits: 0`
  - `Parallel Contributor Poll Limit Exits: 4`
  - `Parallel Contributor Poll Limit: Ordered After Visible: 4`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`
