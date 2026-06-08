# Task 87 Packet 023 Artifact Manifest

- head SHA: `e6b14dfd68c58f3b785179f730b786bc0599fd40`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/023-phase7-methodology-and-closeout/`
- timestamp: `2026-06-08T17:20:00-07:00`
- lane: local PG18 scratch cluster
- database: `postgres`
- socket/port: `/home/peter/.pgrx`, `28818`
- storage surface: existing one-index-per-surface real-corpus benchmark indexes
- suite source packets: packet 021 real10k, packet 022 real50k/real100k

## Artifacts

- `methodology.md`: addresses packet 021 cross-cutting methodology feedback.
- `aggregate-matrix.md`: superseding Phase 7 aggregate matrix.
- `completion-audit.md`: final acceptance audit.
- `hnsw-reloptions-check.log`: first HNSW reloptions lookup by exact expected
  names; returned zero rows.
- `hnsw-reloptions-list.log`: successful HNSW index reloptions listing used to
  identify the current real-corpus HNSW profiles as source-backed, not
  TurboQuant FullLut surfaces.

## HNSW Reloptions Command

```sh
target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --sql "SELECT c.relname, ix.indrelid::regclass AS table_name, c.reloptions FROM pg_class c JOIN pg_index ix ON ix.indexrelid = c.oid JOIN pg_am am ON am.oid = c.relam WHERE am.amname = 'ec_hnsw' ORDER BY c.relname;" --log-output reviews/task-87/023-phase7-methodology-and-closeout/artifacts/hnsw-reloptions-list.log
```

Key result lines from `hnsw-reloptions-list.log`:

- `current_intel_real100k_hnsw_m16_idx`: `{m=16,ef_construction=128,build_source_column=source}`
- `task67_current_shape_real50k_hnsw_m16_idx`: `{m=16,ef_construction=128,build_source_column=source}`
- `task87_phase6_real10k_hnsw_m16_idx`: `{m=16,ef_construction=128}`

None of those HNSW profiles advertises `storage_format=turboquant`.
