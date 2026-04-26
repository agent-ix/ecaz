# Artifact Manifest

Packet: `609-c1-pg18-scale-invariant-drain`

Head SHA: `3fb20730 3dc8d6c12df6a3f914ccef18c49919a7`

Timestamp: `2026-04-24T22:30:00Z`

## `pg18-parallel-5k-default.log`

- packet/topic: `609-c1-pg18-scale-invariant-drain`
- lane: default elected visible tuple emitter
- fixture: `pg18-parallel-scan`, rows=5000, limit=100
- storage format: default scalar `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 5000 --limit 100 --log-output target/pg18-parallel-5k-default.log`
- key result lines:
  - `Limit (actual time=18.486..19.657 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
  - `Heap TIDs Returned: 100`
  - `Parallel Contributor Hidden Publishes: 0`
  - `Parallel Contributor Poll Limit Exits: 0`
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`

## `pg18-parallel-5k-diagnostic.log`

- packet/topic: `609-c1-pg18-scale-invariant-drain`
- lane: contributor diagnostic, `TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1`
- fixture: `pg18-parallel-scan`, rows=5000, limit=100
- storage format: default scalar `ec_hnsw` fixture
- rerank mode: not applicable
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 5000 --limit 100 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-5k-diagnostic.log`
- key result lines:
  - `Limit (actual time=38.702..40.067 rows=100.00 loops=1)`
  - `Bootstrap Expansions: 101`
  - `Elements Scored: 101`
  - `Heap TIDs Returned: 100`
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
  - `candidate_missing_serial_ids=[]`
  - `candidate_extra_ids=[]`
  - `planner-visible Parallel Index Scan validation passed`
