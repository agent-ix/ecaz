# Task 94 Packet 027 Artifact Manifest

- head SHA: `a808ee5c0c6ecd7a3fac9d8fbcf38bfd77dfa3cf`
- task bucket: `reviews/task-94/`
- packet path: `reviews/task-94/027-latency-width-rerun/`
- timestamp: `2026-06-10T16:23:30-07:00`
- lane: local PG18 / Intel AVX2
- fixture/storage: IVF PqFastScan 10k (`task94_local_pqfs10k_roff`) and forced DiskANN grouped-PQ 50k (`task67_local_fullq_50k_diskann`) / `storage_format=pq_fastscan`
- rerank mode: existing fixture settings
- surface isolation: existing task-local IVF fixture; existing DiskANN fixture with `ec_diskann.prefilter_kind=grouped_pq`

## Build And Catalog Provenance

Artifacts:
- `install-ecaz-pg18.log`
- `restart-pg18.log`
- `catalog-before-refresh.log`
- `catalog-refresh.log`

Commands:
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/027-latency-width-rerun/artifacts/install-ecaz-pg18.log dev install ecaz-pg-test --pg 18`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/027-latency-width-rerun/artifacts/restart-pg18.log dev scratch restart --pg 18 --pgrx-home /home/peter/.pgrx`
- `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --sql "SELECT proname, pg_get_function_result(oid) FROM pg_proc WHERE proname IN ('ec_block_kernel_scoring_snapshot','ec_task87_candidate_batch_scoring_snapshot') ORDER BY proname;" --log-output reviews/task-94/027-latency-width-rerun/artifacts/catalog-before-refresh.log`
- `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --log-output reviews/task-94/027-latency-width-rerun/artifacts/catalog-refresh.log --sql "... DROP/CREATE ec_block_kernel_scoring_snapshot() with width columns ..."`

Key results:
- install backend artifact assertion passed
- installed backend SHA: `d5d0a6009e2b9fe9158a40ff88ded13114e2c2403e8778f91098eb75d5fbc3ba`
- `catalog-before-refresh.log` showed the old `ec_block_kernel_scoring_snapshot()` result type without width columns.
- `catalog-refresh.log` records `DROP FUNCTION` / `CREATE FUNCTION` for the widened zero-arg snapshot function. No fixture tables were dropped or recreated.

## Two-Step Rerun

Artifacts:
- `task94-width-latency-rerun-suite.json`
- `suite-audit.log`
- `suite-run.log`
- `suite-status.log`
- `suite-report.log`
- `suite-manifest.json`
- `results.jsonl`
- `results-report.jsonl`
- `fresh-cache-latency-ivf-pqfastscan-10k-batch-on.log`
- `fresh-cache-latency-diskann-pqfastscan-50k-grouped-pq.log`

Commands:
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/027-latency-width-rerun/artifacts/suite-audit.log bench suite audit --config reviews/task-94/027-latency-width-rerun/artifacts/task94-width-latency-rerun-suite.json`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/027-latency-width-rerun/artifacts/suite-run.log bench suite run --config reviews/task-94/027-latency-width-rerun/artifacts/task94-width-latency-rerun-suite.json --artifact-dir reviews/task-94/027-latency-width-rerun/artifacts --manifest-output reviews/task-94/027-latency-width-rerun/artifacts/suite-manifest.json --results-output reviews/task-94/027-latency-width-rerun/artifacts/results.jsonl`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/027-latency-width-rerun/artifacts/suite-status.log bench suite status --manifest reviews/task-94/027-latency-width-rerun/artifacts/suite-manifest.json`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/027-latency-width-rerun/artifacts/suite-report.log bench suite report --manifest reviews/task-94/027-latency-width-rerun/artifacts/suite-manifest.json --results-output reviews/task-94/027-latency-width-rerun/artifacts/results-report.jsonl`

Key results:
- `suite-status.log`: `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- IVF 10k batch-on p50: `2.70 ms` at `nprobe=32`, `4.10 ms` at `nprobe=64`
- DiskANN 50k forced grouped-PQ p50: `15.3 ms` at `list_size=64`, `15.5 ms` at `list_size=128`
- IVF width rows:
  - `nprobe=32`: `width_lt8=15 width_8_15=20 width_16_31=40 width_ge32=9605`
  - `nprobe=64`: `width_lt8=0 width_8_15=0 width_16_31=500 width_ge32=19500`
- DiskANN width rows:
  - `list_size=64`: `width_lt8=970 width_8_15=2531 width_16_31=4873 width_ge32=201`
  - `list_size=128`: `width_lt8=3003 width_8_15=5295 width_16_31=7515 width_ge32=206`

## Diagnostic Full-Matrix Attempt

Artifacts:
- `task94-full-latency-rerun-suite.json`
- `full-suite-run.log`
- `full-suite-manifest.json`

This run was started to rerun every latency step from packet 026, then stopped before completion after it reproduced the packet-026 slow profile on the original matrix cache-state names and entered the long 100k batch-on cell. Treat it as diagnostic-only, not closeout evidence.

Observed diagnostic rows before termination:
- 10k batch-off p50: `41.1 ms` / `67.9 ms`
- 10k batch-on p50: `46.7 ms` / `80.2 ms`
- 25k batch-off p50: `88.3 ms` / `148.9 ms`
- 25k batch-on p50: `103.8 ms` / `185.1 ms`
- 100k batch-off p50: `279.2 ms` / `561.0 ms`

Interpretation:
- The stale CLI/counter issue is fixed by the rebuilt CLI plus catalog refresh; width fields are present.
- The packet-026 latency shift is not explained by stale CLI alone. It reproduces on the original matrix cache-state names, while the fresh-cache two-step rerun is fast. Do not use packet 026 or the diagnostic full-matrix attempt for AC5 closeout.
