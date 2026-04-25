# Artifact Manifest

Packet: `607-c1-pg18-contributor-drain-classification`

Head SHA: `a6b72d66945afef24b51f0935a1461c3a4785051`

Timestamp: `2026-04-25T04:52:53Z`

## `pg18-parallel-contributor-drain-classification-default.log`

- packet/topic: `607-c1-pg18-contributor-drain-classification`
- lane: default elected visible tuple emitter
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-drain-classification-default.log`
- surface: shared-table fixture surface
- key result lines:
  - `Limit (actual time=13.761..14.902 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Duplicate Retires: 0`
  - `Parallel Contributor Output Limit Exits: 0`
  - `Parallel Contributor Poll Limit Exits: 0`
  - `Parallel Contributor Poll Limit: Missing Hidden: 0`
  - `Parallel Contributor Poll Limit: Duplicate Active: 0`
  - `Parallel Contributor Poll Limit: Handoff Ready: 0`
  - `Parallel Contributor Poll Limit: Ordered After Visible: 0`
  - `Parallel Contributor Poll Limit: No Visible Owner: 0`
  - `[pg18-parallel] candidate_missing_serial_ids=[]`
  - `[pg18-parallel] candidate_extra_ids=[]`
  - `[pg18-parallel] planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-contributor-drain-classification-diagnostic.log`

- packet/topic: `607-c1-pg18-contributor-drain-classification`
- lane: contributor diagnostic, `TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1`
- fixture: `pg18-parallel-scan`
- storage format: default `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-drain-classification-diagnostic.log`
- surface: shared-table fixture surface
- key result lines:
  - `Limit (actual time=33.791..34.848 rows=16.00 loops=1)`
  - `Bootstrap Expansions: 17`
  - `Elements Scored: 17`
  - `Heap TIDs Returned: 16`
  - `Parallel Handoffs: Foreign Selected: 0`
  - `Parallel Handoffs: Foreign Head: 0`
  - `Parallel Contributor Hidden Publishes: 8`
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
