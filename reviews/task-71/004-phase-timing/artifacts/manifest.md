# Task 71 / Packet 004 Artifact Manifest

- Head SHA: `bad1027cf`
- Task bucket: `reviews/task-71/`
- Packet path: `reviews/task-71/004-phase-timing/`
- Slice: IVF build phase timing and safe pq_fastscan staging cleanup
- Storage format: `pq_fastscan`
- Rerank mode: `heap_f32`
- Surface: local PG18 probe, isolated one-index-per-table prefixes
- Timestamp: 2026-06-03 America/Los_Angeles

## Current Safe Artifacts

### `install-after-phase-timing-safe.log`

- Command:
  `./target/debug/ecaz --log-file reviews/task-71/004-phase-timing/artifacts/install-after-phase-timing-safe.log dev install ecaz-pg-test --pg 18`
- Result: passed
- Key lines:
  - `Finished installing ecaz`
  - `backend artifact assertion passed`
  - `sha256=867afc0bbe3c12bc4e619a7c77c767198b0706b1d055ce5e2a38f9a1165613ce`

### `clean-before-phase-timing-probe.log`

- Command:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/004-phase-timing/artifacts/clean-before-phase-timing-probe.log dev test ivf-parallel-build-clean --include-probe --probe-workers 8`
- Result: passed
- Key line:
  - `[ivf-clean] dropped 17 prefixes`

### `probe-phase-timing-w1.log` / `probe-load-real10k-w1-after-loader-timing.log`

- Command:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/004-phase-timing/artifacts/probe-phase-timing-w1.log dev test ivf-parallel-build-probe --drop-first --workers 1 --prefix task71_probe_w1_phase_timing`
- Result: passed
- Key lines:
  - `built task71_probe_w1_phase_timing_idx in 517.65ms`
  - `requested_workers=1 workers_launched=1 heap_tuples=10000 index_tuples=10000 heap_ingest_us=91437 train_model_us=275720 stage_build_plan_us=142949 flush_build_plan_us=3363`

### `probe-phase-timing-w8.log` / `probe-load-real10k-w8-after-loader-timing.log`

- Command:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/004-phase-timing/artifacts/probe-phase-timing-w8.log dev test ivf-parallel-build-probe --drop-first --workers 8 --prefix task71_probe_w8_phase_timing`
- Result: passed
- Key lines:
  - `built task71_probe_w8_phase_timing_idx in 404.66ms`
  - `requested_workers=8 workers_launched=7 heap_tuples=10000 index_tuples=10000 heap_ingest_us=35043 train_model_us=258088 stage_build_plan_us=106057 flush_build_plan_us=2212`

## Rejected Rayon Posting Encode Artifacts

These artifacts came from a candidate implementation that was backed out before
`bad1027cf`. They are retained to document the negative result.

### `rejected-rayon-encode/probe-pq-encode-w1.log`

- Result: passed
- Key lines:
  - `built task71_probe_w1_pq_encode_idx in 432.68ms`
  - `heap_ingest_us=91819 train_model_us=280328 stage_build_plan_us=53575 flush_build_plan_us=2269`

### `rejected-rayon-encode/probe-pq-encode-w8.log`

- Result: passed
- Key lines:
  - `built task71_probe_w8_pq_encode_idx in 349.29ms`
  - `heap_ingest_us=31298 train_model_us=258475 stage_build_plan_us=53270 flush_build_plan_us=2274`

### `rejected-rayon-encode/pq-encode-probe/`

- Command:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-71/003-worker-curve/artifacts/pq-encode-probe/suite-run-100k-w1-w8.log bench suite run --config reviews/task-71/003-worker-curve/suite.json --artifact-dir reviews/task-71/003-worker-curve/artifacts/pq-encode-probe --only load-real100k-w1 --only load-real100k-w8`
- Result: completed, but rejected as a performance regression
- Key lines:
  - w1: `built task71_real100k_w1_idx in 2.31s`
  - w1 timing: `heap_ingest_us=876780 train_model_us=501504 stage_build_plan_us=834268 flush_build_plan_us=79090`
  - w8: `built task71_real100k_w8_idx in 9.22s`
  - w8 timing: `heap_ingest_us=1455206 train_model_us=3280731 stage_build_plan_us=4251560 flush_build_plan_us=185780`
