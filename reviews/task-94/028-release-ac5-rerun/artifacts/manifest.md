# Task 94 Packet 028 Artifact Manifest

- head SHA: `5fc436162`
- task bucket: `reviews/task-94/028-release-ac5-rerun/`
- generated: `2026-06-11T00:22:28Z`
- lane: local PG18 pgrx socket, `/home/peter/.pgrx`, port `28818`
- fixture: Task 94 local pq_fastscan matrix, latency-only selected rerun
- storage format: `pq_fastscan`
- quant/rerank mode: `grouped_pq`; IVF batch-off and batch-on; DiskANN forced grouped-PQ
- surface isolation: shared local pgrx tree, one benchmark PostgreSQL instance

## Backend Provenance

- `install-ecaz-pg18-release.log`: `ecaz dev install ecaz-pg-test --pg 18`; installed backend `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`; SHA `dc9b8141751dd3db0d58a10e1bd4d9681e03cf58dabac439305387f1f1cb6646`.
- `restart-pg18-after-release-install.log`: restarted PG18 after release install.
- `catalog-refresh-build-profile.log`: registered `ecaz_build_profile()` in the already-running scratch DB catalog after adding the SQL-visible marker.
- `build-profile-probe.log`: `SELECT ecaz_build_profile()` returned `release`; `ecaz_build_profile` returns `text`; `ec_block_kernel_scoring_snapshot` exposes the width-bucket columns.
- `suite-manifest.json`: suite preflight recorded `backend.build_profile = "release"`, `backend.sha256 = "dc9b8141751dd3db0d58a10e1bd4d9681e03cf58dabac439305387f1f1cb6646"`, and backend path `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`.

## Commands

- `cargo build -p ecaz-cli --bin ecaz`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/028-release-ac5-rerun/artifacts/install-ecaz-pg18-release.log dev install ecaz-pg-test --pg 18`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/028-release-ac5-rerun/artifacts/restart-pg18-after-release-install.log dev scratch restart --pg 18 --pgrx-home /home/peter/.pgrx`
- `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --log-output reviews/task-94/028-release-ac5-rerun/artifacts/catalog-refresh-build-profile.log --sql "CREATE FUNCTION ecaz_build_profile() RETURNS TEXT STRICT STABLE LANGUAGE c AS '$libdir/ecaz', 'ecaz_build_profile_wrapper';"`
- `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --sql "SELECT ecaz_build_profile(); SELECT proname, pg_get_function_result(oid) FROM pg_proc WHERE proname IN ('ecaz_build_profile','ec_block_kernel_scoring_snapshot') ORDER BY proname;" --log-output reviews/task-94/028-release-ac5-rerun/artifacts/build-profile-probe.log`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/028-release-ac5-rerun/artifacts/suite-audit.log bench suite audit --config reviews/task-94/028-release-ac5-rerun/artifacts/task94-release-ac5-latency-suite.json`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/028-release-ac5-rerun/artifacts/suite-run.log bench suite run --config reviews/task-94/028-release-ac5-rerun/artifacts/task94-release-ac5-latency-suite.json --artifact-dir reviews/task-94/028-release-ac5-rerun/artifacts --manifest-output reviews/task-94/028-release-ac5-rerun/artifacts/suite-manifest.json --results-output reviews/task-94/028-release-ac5-rerun/artifacts/results.jsonl --only latency-ivf-pqfastscan-10k-batch-off --only latency-ivf-pqfastscan-10k-batch-on --only latency-ivf-pqfastscan-25k-batch-off --only latency-ivf-pqfastscan-25k-batch-on --only latency-ivf-pqfastscan-100k-batch-off --only latency-ivf-pqfastscan-100k-batch-on --only latency-diskann-pqfastscan-50k-grouped-pq --only latency-diskann-pqfastscan-100k-grouped-pq`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/028-release-ac5-rerun/artifacts/suite-status.log bench suite status --manifest reviews/task-94/028-release-ac5-rerun/artifacts/suite-manifest.json`
- `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-94/028-release-ac5-rerun/artifacts/suite-report.log bench suite report --manifest reviews/task-94/028-release-ac5-rerun/artifacts/suite-manifest.json --results-output reviews/task-94/028-release-ac5-rerun/artifacts/results-report.jsonl`
- `cargo test -p ecaz-cli bench::suite --no-default-features`, captured in `cargo-test-ecaz-cli-suite.log`.

## Key Result Lines

- `suite-status.log`: completed `8`, failed `0`, skipped `6`, dry-run `0`, missing artifacts `0`, stale `0`. The skipped steps are recall steps intentionally excluded from this latency-only rerun.
- IVF batch-on p50 vs batch-off p50 in `results.jsonl`:
  - 10k nprobe 32: `2.79 ms` vs `2.95 ms`; nprobe 64: `4.44 ms` vs `4.81 ms`.
  - 25k nprobe 32: `5.33 ms` vs `5.73 ms`; nprobe 64: `9.46 ms` vs `10.00 ms`.
  - 100k nprobe 32: `16.60 ms` vs `18.50 ms`; nprobe 64: `31.00 ms` vs `34.30 ms`.
- Compared with packet 025 release-baseline p50s, current batch-on changed by `-3.8%`, `-4.1%`, `-5.0%`, `-4.6%`, `-7.3%`, and `-10.4%` across the six IVF cells.
- DiskANN forced grouped-PQ p50s: 50k list_size 64/128 = `23.90 ms` / `18.10 ms`; 100k list_size 64/128 = `40.80 ms` / `32.00 ms`.
- Width-bucket counter rows are present for every batch-on IVF sweep and DiskANN sweep in `results.jsonl`.
