# Artifact Manifest

Packet: `583-c1-parallel-scan-locked-owned-read-take`

Head SHA: `7068834ae7e5da75bfcf64e2a8e1aa0edc079b39`

Generated: `2026-04-24T08:35:36-07:00`

## `pg18-parallel-scan.log`

- Lane: PG18 safe live preflight
- Fixture: `ecaz dev test pg18-parallel-scan` default fixture
- Storage format: default `ec_hnsw`
- Rerank mode: default
- Command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`
- Surface: shared fixture table; ordered serial/candidate ID comparison
- Key result lines:
  - `serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `PostgreSQL can launch workers for the fixture, but did not choose a real Parallel Index Scan; use --expect-parallel once AM planner path activation is ready`
