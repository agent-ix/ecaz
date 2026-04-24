# Artifact Manifest

## pg18-amcanparallel-probe.log

- Head SHA: `e83b8f93f060cc4f5515567420242f3fdfb634dc`
- Packet/topic: `589-c1-pg18-amcanparallel-probe`
- Lane: PG18 local-only `amcanparallel` planner activation probe
- Local source state: temporary uncommitted one-line probe,
  `amroutine.amcanparallel = cfg!(feature = "pg18");`
- Fixture: `pg18_parallel_scan_fixture`, 512 rows, 4 workers, limit 16,
  `ec_hnsw.ef_search = 1000`
- Storage format: default scalar `ec_hnsw` index over `ecvector_ip_ops`,
  created with `WITH (m = 8, ef_construction = 80)`
- Rerank mode: default scalar ordered scan path; no pq-fastscan or rerank
  source override
- Install command:
  `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`
- Diagnostic command:
  `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --diagnose-planner --log-output review/589-c1-pg18-amcanparallel-probe/artifacts/pg18-amcanparallel-probe.log`
- Timestamp: `2026-04-24T11:35:03-07:00`
- Surface: shared-table fixture. The serial ordered query, parallel-candidate
  ordered query, seqscan control, and ordered seqscan control all run against
  the same fixture table; this is not an isolated one-index-per-table surface.
- Artifact: `artifacts/pg18-amcanparallel-probe.log`
- Cleanup: source restored to `amroutine.amcanparallel = false;`, normal PG18
  build reinstalled, and restored install verified with
  `target/pg18-postprobe-normal.log`.

Key result lines:

- `Workers Planned: 4`
- `Workers Launched: 4`
- `ec_hnsw_index_count=1 amcanparallel_seen=true`
- `partial_path_count=1 partial_index_path_count=1 partial_ec_hnsw_index_path_count=1`
- `best_partial_ec_hnsw startup_cost=4015.515 total_cost=4032.795 parallel_workers=4 parallel_aware=true pathkeys=1`
- parallel-candidate ordered JSON plan: `"Node Type": "Index Scan"`
- parallel-candidate ordered JSON plan: `"Parallel Aware": false`
- `[pg18-parallel] serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
- `[pg18-parallel] candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
- `[pg18-parallel] PostgreSQL can launch workers for the fixture, but did not choose a real Parallel Index Scan; use --expect-parallel once AM planner path activation is ready`
