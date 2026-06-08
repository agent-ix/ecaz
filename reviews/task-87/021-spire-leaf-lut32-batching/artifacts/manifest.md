# Task 87 Packet 021 Artifact Manifest

- Head SHA: `56299f37fdce4300dfba11ab5b63f21284adb6bd`
- Task bucket: `reviews/task-87/`
- Packet path: `reviews/task-87/021-spire-leaf-lut32-batching/`
- Timestamp: `2026-06-08T16:22:16-07:00`
- Lane: local PG18 scratch cluster
- Database: `postgres`
- Socket/port: `/home/peter/.pgrx`, `28818`
- Storage format: TurboQuant 4-bit for IVF/SPIRE real10k indexes; HNSW existing real10k profile
- Rerank mode: IVF `heap_f32` rerank width 25; SPIRE rerank width 25
- Isolation: existing one-index-per-surface real10k tables from Task 87 phase 6 prep

## Code Validation

### `test-ec-spire-quantizer.log`

- Command:
  `cargo test --lib am::ec_spire::quantizer --no-default-features --features pg18`
- Result:
  `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 1990 filtered out`

### `test-ec-spire-scan.log`

- Command:
  `cargo test --lib am::ec_spire::scan --no-default-features --features pg18`
- Result:
  `test result: ok. 99 passed; 0 failed; 0 ignored; 0 measured; 1906 filtered out`

### `test-common-candidate-batch.log`

- Command:
  `cargo test --lib am::common::candidate_batch --no-default-features --features pg18`
- Result:
  `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 2001 filtered out`

## Install / SQL Checks

### `install-ecaz-pg-test.log`

- Command:
  `target/debug/ecaz --log-file reviews/task-87/021-spire-leaf-lut32-batching/artifacts/install-ecaz-pg-test.log dev install ecaz-pg-test --pg 18`
- Installed backend:
  `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`
- Installed backend SHA256:
  `b7cdee8d972cd7f45725a8875116a47f89647700d5920d1fe0e42e005bf158c2`

### `counter-function-check.log`

- Command:
  `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --sql "SELECT to_regprocedure('ec_task87_candidate_batch_scoring_reset()') AS reset_fn, to_regprocedure('ec_task87_candidate_batch_scoring_snapshot()') AS snapshot_fn;" --log-output reviews/task-87/021-spire-leaf-lut32-batching/artifacts/counter-function-check.log`
- Result:
  both `ec_task87_candidate_batch_scoring_reset()` and `ec_task87_candidate_batch_scoring_snapshot()` registered.

## Suite

### Config

- Suite config:
  `reviews/task-87/021-spire-leaf-lut32-batching/phase7-real10k-counter-suite.json`
- Command:
  `target/debug/ecaz bench suite run --config reviews/task-87/021-spire-leaf-lut32-batching/phase7-real10k-counter-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-87/021-spire-leaf-lut32-batching/artifacts/real10k-run-manifest.json --results-output reviews/task-87/021-spire-leaf-lut32-batching/artifacts/real10k-results.jsonl --log-file reviews/task-87/021-spire-leaf-lut32-batching/artifacts/real10k-run.log`
- Suite status:
  `completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

### Key Result Lines

- IVF recall off/on:
  - off: `recall@k=1.0000`, mean query time `19.96 ms`
  - on: `recall@k=1.0000`, mean query time `17.36 ms`
- IVF latency off:
  `p50=19.3 ms`, `p95=20.9 ms`, `p99=22.8 ms`
- IVF latency on:
  `p50=16.7 ms`, `p95=18.0 ms`, `p99=21.1 ms`
- IVF candidate-batch-on counters:
  `surface=ivf flushes=8000 candidates=2000000 elapsed_ms=2294.054086 lut32_flushes=7800 lut32_candidates=1996800`
- SPIRE pipeline off:
  `latency_p50=17.686 ms`, `latency_p95=20.579 ms`, `latency_p99=22.502 ms`, `recall@k=1.0000`
- SPIRE pipeline on:
  `latency_p50=15.413 ms`, `latency_p95=17.951 ms`, `latency_p99=22.600 ms`, `recall@k=1.0000`
- SPIRE candidate-batch-on counters:
  `surface=spire flushes=4800 candidates=1551640 elapsed_ms=1793.231978 lut32_flushes=4800 lut32_candidates=1551640`
- SPIRE candidate-batch-off counters:
  `surface=spire flushes=0 candidates=0 elapsed_ms=0.000000 lut32_flushes=0 lut32_candidates=0`
- HNSW latency:
  `p50=4.59 ms`, `p95=6.44 ms`, `p99=7.41 ms`
- HNSW counters:
  `surface=hnsw flushes=0 candidates=0 elapsed_ms=0.000000 lut32_flushes=0 lut32_candidates=0`

## Artifact Files

- `real10k-run-manifest.json`
- `real10k-results.jsonl`
- `real10k-run.log`
- `real10k-status.log`
- `counter-function-check.log`
- `install-ecaz-pg-test.log`
- `test-ec-spire-quantizer.log`
- `test-ec-spire-scan.log`
- `test-common-candidate-batch.log`
- `run/precheck-host.log`
- `run/recall-real10k-ivf-candidate-batch-off.log`
- `run/recall-real10k-ivf-candidate-batch-on.log`
- `run/latency-real10k-ivf-candidate-batch-off.log`
- `run/latency-real10k-ivf-candidate-batch-on.log`
- `run/storage-real10k-ivf.log`
- `run/pipeline-real10k-spire-candidate-batch-off.log`
- `run/pipeline-real10k-spire-candidate-batch-on.log`
- `run/storage-real10k-spire.log`
- `run/latency-real10k-hnsw-candidate-batch-on.log`
