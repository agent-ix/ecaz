# Artifact Manifest

Packet: `611-c1-pg18-high-dimensional-parallel-fixture`

Head SHA: `8ae8fe4a0e2a0b8446dd75b88cf0a696183d4bbc`

Timestamp: `2026-04-25T06:05:32Z`

## `pg18-parallel-50k-dim16-random-default.log`

- packet/topic: `611-c1-pg18-high-dimensional-parallel-fixture`
- lane: default elected visible tuple emitter, LIMIT 100
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-random-default.log`
- surface: shared-table fixture surface
- key result lines:
  - `[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500`
  - `Limit (actual time=15.930..16.944 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
  - `Heap TIDs Returned: 100`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Publish: Missing Hidden: 0`
  - `Parallel Contributor Publish: Duplicate Active: 0`
  - `Parallel Contributor Publish: Handoff Ready: 0`
  - `Parallel Contributor Publish: Ordered After Visible: 0`
  - `Parallel Contributor Publish: No Visible Owner: 0`
  - `Parallel Contributor Duplicate Retires: 0`
  - `Parallel Contributor Output Limit Exits: 0`
  - `Parallel Contributor Poll Limit Exits: 0`
  - `Parallel Contributor Poll Limit: No Visible Owner: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-50k-dim16-random-diagnostic.log`

- packet/topic: `611-c1-pg18-high-dimensional-parallel-fixture`
- lane: contributor diagnostic, `TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1`, LIMIT 100
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-random-diagnostic.log`
- surface: shared-table fixture surface
- key result lines:
  - `[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500`
  - `Limit (actual time=32.676..34.372 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
  - `Heap TIDs Returned: 100`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 8`
  - `Parallel Contributor Publish: Missing Hidden: 0`
  - `Parallel Contributor Publish: Duplicate Active: 4`
  - `Parallel Contributor Publish: Handoff Ready: 0`
  - `Parallel Contributor Publish: Ordered After Visible: 0`
  - `Parallel Contributor Publish: No Visible Owner: 4`
  - `Parallel Contributor Duplicate Retires: 4`
  - `Parallel Contributor Output Limit Exits: 0`
  - `Parallel Contributor Poll Limit Exits: 4`
  - `Parallel Contributor Poll Limit: Missing Hidden: 0`
  - `Parallel Contributor Poll Limit: Duplicate Active: 0`
  - `Parallel Contributor Poll Limit: Handoff Ready: 0`
  - `Parallel Contributor Poll Limit: Ordered After Visible: 0`
  - `Parallel Contributor Poll Limit: No Visible Owner: 4`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`
