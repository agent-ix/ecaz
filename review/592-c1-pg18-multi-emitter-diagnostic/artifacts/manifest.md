# Artifact Manifest

## Packet

- packet: `592-c1-pg18-multi-emitter-diagnostic`
- head SHA: `f8e34609125c66f99b2b1b6e66c8372c44dc0e4b`
- timestamp: `2026-04-24T17:05:31-07:00`

## Artifacts

### `pg18-parallel-multi-emitter-default.log`

- lane: PG18 default elected-emitter validation, leader participation on
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-multi-emitter-diagnostic-default.log`
- key lines:
  - `env=[]`
  - `parallel_leader_participation=on`
  - `Workers Launched: 4`
  - `Parallel Index Scan using pg18_parallel_scan_fixture_idx`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-multi-emitter-default-leader-off.log`

- lane: PG18 default elected-emitter validation, leader participation off
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --disable-parallel-leader-participation --log-output target/pg18-parallel-multi-emitter-diagnostic-default-leader-off.log`
- key lines:
  - `env=[]`
  - `parallel_leader_participation=off`
  - `Workers Launched: 4`
  - `Parallel Index Scan using pg18_parallel_scan_fixture_idx`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-multi-emitter-enabled.log`

- lane: PG18 diagnostic multi-emitter validation, leader participation on
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-multi-emitter-diagnostic-enabled.log`
- key lines:
  - `env=["TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC"]`
  - `parallel_leader_participation=on`
  - `Workers Launched: 4`
  - `Parallel Index Scan using pg18_parallel_scan_fixture_idx`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[379, 93, 177, 472, 473, 378, 176, 71, 172, 280, 57, 366, 258, 82, 78, 459]`
  - `validation failed`

### `pg18-parallel-multi-emitter-enabled-leader-off.log`

- lane: PG18 diagnostic multi-emitter validation, leader participation off
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --disable-parallel-leader-participation --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-multi-emitter-diagnostic-enabled-leader-off.log`
- key lines:
  - `env=["TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC"]`
  - `parallel_leader_participation=off`
  - `Workers Launched: 4`
  - `Parallel Index Scan using pg18_parallel_scan_fixture_idx`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[379, 177, 473, 472, 378, 93, 57, 366, 258, 172, 280, 176, 71, 82, 459, 284]`
  - `validation failed`
