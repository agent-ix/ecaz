# Artifact Manifest

Packet: `608-c1-pg18-visible-emitter-serial-ef-search`

Head SHA: `74f7df0cc629673552c725848b24f91bbf54b681`

Timestamp: `2026-04-25T05:25:42Z`

## `pg18-parallel-limit33-visible-full-ef.log`

- packet/topic: `608-c1-pg18-visible-emitter-serial-ef-search`
- lane: default elected visible tuple emitter, LIMIT 33
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --limit 33 --log-output target/pg18-parallel-limit33-visible-full-ef.log`
- surface: shared-table fixture surface
- key result lines:
  - `Limit (actual time=14.556..15.766 rows=33.00 loops=1)`
  - `Bootstrap Expansions: 34`
  - `Elements Scored: 34`
  - `Heap TIDs Returned: 33`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Poll Limit: No Visible Owner: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-limit64-visible-full-ef.log`

- packet/topic: `608-c1-pg18-visible-emitter-serial-ef-search`
- lane: default elected visible tuple emitter, LIMIT 64
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --limit 64 --log-output target/pg18-parallel-limit64-visible-full-ef.log`
- surface: shared-table fixture surface
- key result lines:
  - `Limit (actual time=15.588..16.593 rows=64.00 loops=1)`
  - `Bootstrap Expansions: 65`
  - `Elements Scored: 65`
  - `Heap TIDs Returned: 64`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Poll Limit: No Visible Owner: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`
