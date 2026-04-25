# Artifact Manifest

Packet: `612-c1-pg18-contributor-no-owner-drop`

Head SHA: `9de975894acf48f407d93dbd7aed0f662e898028`

Timestamp: `2026-04-25T06:27:26Z`

## `pg18-parallel-50k-dim16-no-owner-drop-default.log`

- packet/topic: `612-c1-pg18-contributor-no-owner-drop`
- lane: default elected visible tuple emitter, LIMIT 100
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-no-owner-drop-default.log`
- surface: shared-table fixture surface
- key result lines:
  - `[pg18-parallel] env=[]`
  - `[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500`
  - `Limit (actual time=13.316..14.487 rows=100.00 loops=1)`
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
  - `Parallel Contributor No Visible Owner Drops: 0`
  - `Parallel Contributor Duplicate Retires: 0`
  - `Parallel Contributor Output Limit Exits: 0`
  - `Parallel Contributor Poll Limit Exits: 0`
  - `Parallel Contributor Poll Limit: No Visible Owner: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-50k-dim16-no-owner-drop-diagnostic.log`

- packet/topic: `612-c1-pg18-contributor-no-owner-drop`
- lane: contributor diagnostic, `TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1`, LIMIT 100
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-no-owner-drop-diagnostic.log`
- surface: shared-table fixture surface
- key result lines:
  - `[pg18-parallel] env=["TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC"]`
  - `[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500`
  - `Limit (actual time=41.710..43.188 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
  - `Heap TIDs Returned: 100`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 260`
  - `Parallel Contributor Publish: Missing Hidden: 0`
  - `Parallel Contributor Publish: Duplicate Active: 4`
  - `Parallel Contributor Publish: Handoff Ready: 0`
  - `Parallel Contributor Publish: Ordered After Visible: 0`
  - `Parallel Contributor Publish: No Visible Owner: 256`
  - `Parallel Contributor No Visible Owner Drops: 252`
  - `Parallel Contributor Duplicate Retires: 4`
  - `Parallel Contributor Output Limit Exits: 4`
  - `Parallel Contributor Poll Limit Exits: 0`
  - `Parallel Contributor Poll Limit: Missing Hidden: 0`
  - `Parallel Contributor Poll Limit: Duplicate Active: 0`
  - `Parallel Contributor Poll Limit: Handoff Ready: 0`
  - `Parallel Contributor Poll Limit: Ordered After Visible: 0`
  - `Parallel Contributor Poll Limit: No Visible Owner: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`
