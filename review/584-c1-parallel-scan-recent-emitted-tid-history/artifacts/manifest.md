# Artifact Manifest

Packet: `584-c1-parallel-scan-recent-emitted-tid-history`

Head SHA: `81ed9368b466c697ee4a34dd3ddc659b61acf68c`

Generated: `2026-04-24T08:55:49-07:00`

## `pg18-parallel-scan.log`

- Lane: PG18 safe live preflight
- Fixture: `ecaz dev test pg18-parallel-scan` default fixture
- Storage format: default `ec_hnsw`
- Rerank mode: default
- Command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`
- Surface: shared fixture table; ordered serial/candidate ID comparison
- Key result lines:
  - `Workers Planned: 4`
  - `Workers Launched: 4`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `PostgreSQL can launch workers for the fixture, but did not choose a real Parallel Index Scan; use --expect-parallel once AM planner path activation is ready`
