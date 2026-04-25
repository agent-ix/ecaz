# Artifact Manifest

## pg18-parallel-50k-dim16-rank-relation-contributor.log

- head SHA: `d484e6fa44a672538e62c1fbdf3195803e341987`
- packet/topic: `617-c1-pg18-contributor-rank-relation-counters`
- lane: PG18 contributor diagnostic
- fixture: `rows=50000`, `workers=4`, `dimensions=16`, randomized embeddings, `limit=100`, `ef_search=500`
- storage format: default scalar `ec_hnsw` index on `ecvector`
- rerank mode: none
- isolated one-index-per-table surface: yes
- shared-table surface: no
- command:
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-rank-relation-contributor.log`
- timestamp: `2026-04-25T13:42:52Z`
- key cited lines:
  - `[pg18-parallel] env=["TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC"]`
  - `[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500`
  - `Limit (actual time=148.389..156.944 rows=100.00 loops=1)`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 260`
  - `Parallel Contributor Publish: Duplicate Active: 8`
  - `Parallel Contributor Publish: Handoff Ready: 0`
  - `Parallel Contributor Publish: Ordered After Visible: 252`
  - `Parallel Contributor Publish: No Visible Owner: 0`
  - `Parallel Contributor Publish Rank: Before Visible: 0`
  - `Parallel Contributor Publish Rank: Equal Visible: 4`
  - `Parallel Contributor Publish Rank: After Visible: 252`
  - `Parallel Contributor Publish Rank: Missing Visible: 0`
  - `Parallel Contributor Duplicate Active Drops: 4`
  - `Parallel Contributor Ordered After Visible Drops: 248`
  - `Parallel Visible Owner Lookahead Publishes: 100`
  - `Parallel Contributor Output Limit Exits: 4`
  - `Parallel Contributor Poll Limit Exits: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`
