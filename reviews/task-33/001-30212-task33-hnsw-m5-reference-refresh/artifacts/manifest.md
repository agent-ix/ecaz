# Artifact Manifest: Task 33 HNSW M5 Reference Refresh Scaffold

- head SHA: `586a4d8bf2c8e74df4e1c67699a9cd93c1cc3d08`
- task bucket: `reviews/task-33`
- packet path: `reviews/task-33/001-30212-task33-hnsw-m5-reference-refresh`
- lane: HNSW M5 real50K worker-sweep scaffold
- timestamp: `2026-05-30T15:54:06Z`
- hardware: Apple M5 local development machine when executed
- PostgreSQL: PG18 on socket `/Users/peter/.pgrx`, port `28818`
- fixture: `data/task31_m5_dbpedia_staged/ec_hnsw_real_50k_*`
- isolation/shared-table surface: one Task 33 corpus prefix with one index per
  requested worker count during the raw SQL sweep

## Artifacts

| Artifact | Purpose |
| --- | --- |
| `../task33-hnsw-m5-reference-refresh.packet.json` | suite config |
| `task33_hnsw_m5_worker_sweep.sql` | packet-local SQL for worker-count build sweep |
| `audit.log` | suite audit output |
| `dry-run-suite-manifest.json` | dry-run expansion of suite commands |

## Commands

```sh
/Users/peter/.cargo/bin/ecaz --log-file reviews/task-33/001-30212-task33-hnsw-m5-reference-refresh/artifacts/audit.log bench suite audit --config reviews/task-33/001-30212-task33-hnsw-m5-reference-refresh/task33-hnsw-m5-reference-refresh.packet.json
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-33/001-30212-task33-hnsw-m5-reference-refresh/task33-hnsw-m5-reference-refresh.packet.json --manifest-output reviews/task-33/001-30212-task33-hnsw-m5-reference-refresh/artifacts/dry-run-suite-manifest.json
```

## Results

No benchmark measurements were executed in this scaffold packet. This packet
only validates and publishes the suite configuration for the next Task 33 run.

## Validation

- `ecaz bench suite audit`: passed, `5` steps
- `ecaz bench suite run --dry-run`: wrote `dry-run-suite-manifest.json`
- `jq empty` on the suite config and dry-run manifest: passed
- `git diff --check`: passed
