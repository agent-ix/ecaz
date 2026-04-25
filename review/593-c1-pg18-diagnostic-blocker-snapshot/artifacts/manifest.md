# Artifact Manifest

## Packet

- packet: `593-c1-pg18-diagnostic-blocker-snapshot`
- head SHA: `b43a4bee3dee6703bddc922afb6592a02ffe9f46`
- timestamp: `2026-04-24T17:24:38-07:00`

## Artifacts

### `pg18-parallel-snapshot-blocker-default.log`

- lane: PG18 default elected-emitter validation, leader participation on
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-snapshot-blocker-default.log`
- key lines:
  - `env=[]`
  - `Workers Launched: 4`
  - `Parallel Index Scan using pg18_parallel_scan_fixture_idx`
  - `next_runtime_blocker=PG18 planner-visible Parallel Index Scan is enabled with one elected tuple emitter; rank-compatible multi-emitter Gather Merge output remains the next runtime step`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-snapshot-blocker-diagnostic.log`

- lane: PG18 diagnostic multi-emitter validation, leader participation on
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-snapshot-blocker-diagnostic.log`
- key lines:
  - `env=["TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC"]`
  - `Workers Launched: 4`
  - `Parallel Index Scan using pg18_parallel_scan_fixture_idx`
  - `next_runtime_blocker=PG18 diagnostic multi-emitter env is enabled; direct multi-emitter output remains rank-incompatible with Gather Merge`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[473, 379, 93, 177, 472, 378, 280, 366, 57, 176, 258, 71, 172, 280, 176, 284]`
  - `validation failed`
