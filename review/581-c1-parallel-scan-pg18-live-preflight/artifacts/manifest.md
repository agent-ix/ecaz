# Artifact Manifest: Parallel Scan PG18 Live Preflight

Artifact: `pg18-parallel-scan.log`
- Head SHA: `96a2d87de2e03733acb79d28e0e299cdadbd2552`
- Packet/topic: `581-c1-parallel-scan-pg18-live-preflight`
- Lane / fixture / storage format / rerank mode: Task 18 PG18 live planner preflight / `pg18_parallel_scan_fixture` / `ecvector` scalar `ec_hnsw` / no rerank
- Command: `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`
- Timestamp: `2026-04-23T21:33:40-07:00`
- Table surface: isolated one-index-per-table fixture
- Key result lines:
  - `[pg18-parallel] rows=512 workers=4 limit=16 ef_search=1000`
  - `->  Index Scan using pg18_parallel_scan_fixture_idx on pg18_parallel_scan_fixture`
  - `[pg18-parallel] serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `[pg18-parallel] candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
  - `[pg18-parallel] PostgreSQL did not choose a real Parallel Index Scan; use --expect-parallel once planner path activation is ready`
