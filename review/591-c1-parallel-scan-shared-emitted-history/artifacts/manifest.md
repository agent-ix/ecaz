# Artifact Manifest

## Packet

- packet: `591-c1-parallel-scan-shared-emitted-history`
- head SHA: `d3357bd4caf85f0d0754b8597fd45b6459f4968f`
- timestamp: `2026-04-24T16:40:16-07:00`

## Artifacts

### `pg18-parallel-shared-emitted-default.log`

- lane: PG18 planner-visible parallel scan with shared emitted-history runtime, leader participation on
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-shared-emitted-default.log`
- key lines:
  - `Workers Planned: 4`
  - `Workers Launched: 4`
  - `Parallel Index Scan using pg18_parallel_scan_fixture_idx`
  - `parallel_leader_participation=on`
  - `partial_ec_hnsw_index_path_count=1`
  - `best_partial_ec_hnsw startup_cost=0.000 total_cost=1021.159 parallel_workers=4 parallel_aware=true pathkeys=1`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `planner-visible Parallel Index Scan validation passed`

### `pg18-parallel-shared-emitted-leader-off.log`

- lane: PG18 planner-visible parallel scan with shared emitted-history runtime, leader participation off
- fixture: `ecaz-cli dev test pg18-parallel-scan`, 512-row `pg18_parallel_scan_fixture`, `workers=4`, `limit=16`, `ef_search=1000`
- storage format: default `ec_hnsw` fixture index, one fixture index on one fixture table
- rerank mode: not applicable for this scalar fixture
- surface: isolated one-index-per-table validation cluster
- command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --disable-parallel-leader-participation --log-output target/pg18-parallel-shared-emitted-leader-off.log`
- key lines:
  - `Workers Planned: 4`
  - `Workers Launched: 4`
  - `Parallel Index Scan using pg18_parallel_scan_fixture_idx`
  - `parallel_leader_participation=off`
  - `partial_ec_hnsw_index_path_count=1`
  - `best_partial_ec_hnsw startup_cost=0.000 total_cost=1021.159 parallel_workers=4 parallel_aware=true pathkeys=1`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `planner-visible Parallel Index Scan validation passed`
