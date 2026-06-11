# Task 101 Packet 004 Artifact Manifest

- head SHA: `5fc436162`
- task bucket: `reviews/task-101/004-release-ac5-rerun/`
- generated: `2026-06-11T00:22:28Z`
- source evidence packet: `reviews/task-94/028-release-ac5-rerun/`
- lane: local PG18 pgrx socket, `/home/peter/.pgrx`, port `28818`
- fixture: Task 94/101 local pq_fastscan latency matrix, latency-only selected rerun
- storage format: `pq_fastscan`
- quant/rerank mode: `grouped_pq`; IVF batch-off and batch-on; DiskANN forced grouped-PQ
- surface isolation: shared local pgrx tree, one benchmark PostgreSQL instance

## Packet-Local Copies

The following files are copied from `reviews/task-94/028-release-ac5-rerun/artifacts/` so this Task 101 AC5 closeout is self-contained:

- `suite-manifest.json`
- `results.jsonl`
- `results-report.jsonl`
- `suite-report.log`
- `suite-status.log`
- `build-profile-probe.log`
- `cargo-test-ecaz-cli-suite.log`
- `task94-release-ac5-latency-suite.json`

## Backend Provenance

- `build-profile-probe.log`: `SELECT ecaz_build_profile()` returned `release`.
- `suite-manifest.json`: suite preflight recorded `backend.build_profile = "release"`, `backend.sha256 = "dc9b8141751dd3db0d58a10e1bd4d9681e03cf58dabac439305387f1f1cb6646"`, and backend path `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`.

## Key Result Lines

- `suite-status.log`: completed `8`, failed `0`, skipped `6`, dry-run `0`, missing artifacts `0`, stale `0`. The skipped steps are recall steps intentionally excluded from this latency-only rerun.
- `cargo-test-ecaz-cli-suite.log`: `cargo test -p ecaz-cli bench::suite --no-default-features` passed, `46 passed`.
- IVF batch-on p50 vs batch-off p50 in `results.jsonl`:
  - 10k nprobe 32: `2.79 ms` vs `2.95 ms`; nprobe 64: `4.44 ms` vs `4.81 ms`.
  - 25k nprobe 32: `5.33 ms` vs `5.73 ms`; nprobe 64: `9.46 ms` vs `10.00 ms`.
  - 100k nprobe 32: `16.60 ms` vs `18.50 ms`; nprobe 64: `31.00 ms` vs `34.30 ms`.
- DiskANN forced grouped-PQ p50s: 50k list_size 64/128 = `23.90 ms` / `18.10 ms`; 100k list_size 64/128 = `40.80 ms` / `32.00 ms`.
- Width-bucket counter rows are present for every batch-on IVF sweep and DiskANN sweep in `results.jsonl`.
