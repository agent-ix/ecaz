# Artifact Manifest

## pg18-parallel-pathlist-diagnostics.log

- Head SHA: `70f662a83dff5551ef41290b2f24baba3f9afdc7`
- Packet/topic: `588-c1-pg18-parallel-pathlist-diagnostics`
- Lane: PG18 parallel planner pathlist diagnostics
- Fixture: `pg18_parallel_scan_fixture`, 512 rows, 4 workers, limit 16,
  `ec_hnsw.ef_search = 1000`
- Storage format: default scalar `ec_hnsw` index over `ecvector_ip_ops`,
  created with `WITH (m = 8, ef_construction = 80)`
- Rerank mode: default scalar ordered scan path; no pq-fastscan or rerank
  source override
- Command used:
  `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --diagnose-planner --log-output review/588-c1-pg18-parallel-pathlist-diagnostics/artifacts/pg18-parallel-pathlist-diagnostics.log`
- Timestamp: `2026-04-24T11:32:21-07:00`
- Surface: shared-table fixture. The serial ordered query, parallel-candidate
  ordered query, seqscan control, and ordered seqscan control all run against
  the same fixture table; this is not an isolated one-index-per-table surface.
- Artifact: `artifacts/pg18-parallel-pathlist-diagnostics.log`

Key result lines:

- `Workers Planned: 4`
- `Workers Launched: 4`
- `modeled_startup_cost=4015.515 modeled_total_cost=4015.515 modeled_run_cost=0.000 startup_fraction=1.000000 modeled_selectivity=1.000 modeled_correlation=0.000`
- `PG18 planner pathlist snapshot after parallel-candidate ordered plan:`
- `hook_registered=true observed=true relid=1 consider_parallel=true rel_parallel_workers=4`
- `ec_hnsw_index_count=1 amcanparallel_seen=false`
- `path_count=1 index_path_count=1 ec_hnsw_index_path_count=1`
- `partial_path_count=1 partial_index_path_count=0 partial_ec_hnsw_index_path_count=0`
- `best_plain_ec_hnsw startup_cost=4015.515 total_cost=4036.635 parallel_workers=0 pathkeys=1`
- `best_partial_ec_hnsw startup_cost=NULL total_cost=NULL parallel_workers=NULL parallel_aware=NULL pathkeys=NULL`
- parallel ordered seqscan control JSON plan: `"Node Type": "Gather Merge"`
- parallel ordered seqscan control JSON plan child seqscan:
  `"Parallel Aware": true`
- `[pg18-parallel] serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
- `[pg18-parallel] candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
- `[pg18-parallel] PostgreSQL can launch workers for the fixture, but did not choose a real Parallel Index Scan; use --expect-parallel once AM planner path activation is ready`
