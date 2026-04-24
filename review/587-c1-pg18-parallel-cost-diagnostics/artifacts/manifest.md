# Artifact Manifest

## pg18-parallel-cost-diagnostics.log

- Head SHA: `3ee1a522a366269a1156dd0a1f21a284f7eb369e`
- Packet/topic: `587-c1-pg18-parallel-cost-diagnostics`
- Lane: PG18 parallel planner activation diagnostics
- Fixture: `pg18_parallel_scan_fixture`, 512 rows, 4 workers, limit 16,
  `ec_hnsw.ef_search = 1000`
- Storage format: default scalar `ec_hnsw` index over `ecvector_ip_ops`,
  created with `WITH (m = 8, ef_construction = 80)`
- Rerank mode: default scalar ordered scan path; no pq-fastscan or rerank
  source override
- Command used:
  `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --diagnose-planner --log-output review/587-c1-pg18-parallel-cost-diagnostics/artifacts/pg18-parallel-cost-diagnostics.log`
- Timestamp: `2026-04-24T11:10:56-07:00`
- Surface: shared-table fixture. The serial ordered query, parallel-candidate
  ordered query, seqscan control, and ordered seqscan control all run against
  the same fixture table; this is not an isolated one-index-per-table surface.
- Artifact: `artifacts/pg18-parallel-cost-diagnostics.log`

Key result lines:

- `Workers Planned: 4`
- `Workers Launched: 4`
- `ec_hnsw cost snapshot:`
- `modeled_startup_cost=4015.515 modeled_total_cost=4015.515 modeled_run_cost=0.000 startup_fraction=1.000000 modeled_selectivity=1.000 modeled_correlation=0.000`
- parallel-candidate ordered JSON plan: `"Node Type": "Index Scan"`
- parallel-candidate ordered JSON plan: `"Parallel Aware": false`
- parallel ordered seqscan control JSON plan: `"Node Type": "Gather Merge"`
- parallel ordered seqscan control JSON plan child seqscan:
  `"Parallel Aware": true`
- `[pg18-parallel] serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
- `[pg18-parallel] candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]`
- `[pg18-parallel] PostgreSQL can launch workers for the fixture, but did not choose a real Parallel Index Scan; use --expect-parallel once AM planner path activation is ready`
