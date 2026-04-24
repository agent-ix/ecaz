# Artifact Manifest

Packet: `585-c1-pg18-parallel-preflight-local-gucs`

Head SHA: `663ad81aa14d021ed9e80e7511add522ebeefdaa`

Generated: `2026-04-24T09:01:44-07:00`

## `pg18-parallel-scan.log`

- Lane: PG18 safe live preflight
- Fixture: `ecaz dev test pg18-parallel-scan` default fixture
- Storage format: default `ec_hnsw`
- Rerank mode: default
- Command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`
- Surface: shared fixture table; ordered serial/candidate ID comparison plus parallel seqscan control plan
- Key result lines:
  - `Workers Planned: 4`
  - `Workers Launched: 4`
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `PostgreSQL can launch workers for the fixture, but did not choose a real Parallel Index Scan; use --expect-parallel once AM planner path activation is ready`
