# Artifact Manifest

Packet: `610-c1-pg18-contributor-publish-classification`

Head SHA: `e27e7f4e8c62f56ccbb94b46312f5b0fe1cd3fd0`

Timestamp: `2026-04-25T05:55:12Z`

## `pg18-parallel-limit64-publish-classification-default.log`

- packet/topic: `610-c1-pg18-contributor-publish-classification`
- lane: default elected visible tuple emitter, LIMIT 64
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --limit 64 --log-output target/pg18-parallel-limit64-publish-classification-default.log`
- surface: shared-table fixture surface
- key result lines:
  - `Limit (actual time=15.243..15.883 rows=64.00 loops=1)`
  - `Bootstrap Expansions: 65`
  - `Elements Scored: 65`
  - `Heap TIDs Returned: 64`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Publish: Missing Hidden: 0`
  - `Parallel Contributor Publish: Duplicate Active: 0`
  - `Parallel Contributor Publish: Handoff Ready: 0`
  - `Parallel Contributor Publish: Ordered After Visible: 0`
  - `Parallel Contributor Publish: No Visible Owner: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-limit64-publish-classification-diagnostic.log`

- packet/topic: `610-c1-pg18-contributor-publish-classification`
- lane: contributor diagnostic, `TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1`, LIMIT 64
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --limit 64 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-limit64-publish-classification-diagnostic.log`
- surface: shared-table fixture surface
- key result lines:
  - `Limit (actual time=33.829..35.093 rows=64.00 loops=1)`
  - `Bootstrap Expansions: 65`
  - `Elements Scored: 65`
  - `Heap TIDs Returned: 64`
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
