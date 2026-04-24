# Artifact Manifest: Parallel Scan PG18 Worker Control Preflight

Artifact: `pg18-parallel-scan.log`
- Head SHA: `0b65bf9cd4a43f280ae5bd8ee27e47c5201c1f75`
- Packet/topic: `582-c1-parallel-scan-pg18-worker-control-preflight`
- Lane / fixture / storage format / rerank mode: Task 18 PG18 live planner preflight / `pg18_parallel_scan_fixture` / `ecvector` scalar `ec_hnsw` / no rerank
- Command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`
- Timestamp: `2026-04-24T06:55:19-07:00`
- Table surface: isolated one-index-per-table fixture
- Key result lines:
  - `[pg18-parallel] rows=512 workers=4 limit=16 ef_search=1000`
  - `->  Index Scan using pg18_parallel_scan_fixture_idx on pg18_parallel_scan_fixture`
  - `Gather (actual time=0.457..7.406 rows=512.00 loops=1)`
  - `Workers Planned: 4`
  - `Workers Launched: 4`
  - `->  Parallel Seq Scan on pg18_parallel_scan_fixture`
  - `[pg18-parallel] serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `[pg18-parallel] candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `[pg18-parallel] PostgreSQL can launch workers for the fixture, but did not choose a real Parallel Index Scan; use --expect-parallel once AM planner path activation is ready`
