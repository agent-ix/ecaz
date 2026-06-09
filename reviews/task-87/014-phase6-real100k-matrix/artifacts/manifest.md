# Task 87 Packet 014 Artifact Manifest

- head SHA: `85a238c38dcc30612af20edcf916f033632f8e72`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/014-phase6-real100k-matrix/`
- timestamp: `2026-06-08T23:20:00Z`
- scope: Phase 6 real100k measurement slice
- lane / fixture / storage format / rerank mode: local PG18; real100k; HNSW existing TurboQuant surface; IVF existing TurboQuant surface; SPIRE existing TurboQuant surface; IVF/SPIRE rerank width 25
- isolated one-index-per-table vs shared-table surfaces: reused one-AM HNSW, IVF, and SPIRE tables; each table has one ANN index plus btree primary key

## Suite Config

### `phase6-real100k-suite.json`

- source: copied from `reviews/task-87/012-phase6-suite-prep/phase6-suite.json`
  with artifact paths rewritten to `reviews/task-87/014-phase6-real100k-matrix/artifacts/run`.
- command: `target/debug/ecaz bench suite audit --config reviews/task-87/014-phase6-real100k-matrix/phase6-real100k-suite.json`
- result: passed
- key cited line:
  - `[suite:task87-phase6-candidate-batch-matrix] audit passed: 41 steps`

## Artifacts

### `run/truth-real100k-spire-generate.log`

- command: `target/debug/ecaz bench recall --database postgres --host /home/peter/.pgrx --port 28818 --prefix task74_intel_spire_highrecall_tg128_b0 --profile ec_spire --k 10 --sweep 24 --rerank-width 25 --queries-limit 100 --bits 4 --seed 42 --force-index --session-guc ec_spire.candidate_batch_scoring=off --truth-cache-file reviews/task-87/014-phase6-real100k-matrix/artifacts/run/truth-real100k-spire-k10.json --log-output reviews/task-87/014-phase6-real100k-matrix/artifacts/run/truth-real100k-spire-generate.log`
- result: passed
- key cited lines:
  - `ground truth in 17.88s`
  - `wrote ground truth cache reviews/task-87/014-phase6-real100k-matrix/artifacts/run/truth-real100k-spire-k10.json`
  - `recall@k 0.9100`
  - `mean q-time 448.61 ms`

### `real100k-run.log`

- command: `target/debug/ecaz bench suite run --config reviews/task-87/014-phase6-real100k-matrix/phase6-real100k-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --only recall-real100k-hnsw-candidate-batch-off --only recall-real100k-hnsw-candidate-batch-on --only latency-real100k-hnsw-candidate-batch-off --only latency-real100k-hnsw-candidate-batch-on --only storage-real100k-hnsw --only recall-real100k-ivf-candidate-batch-off --only recall-real100k-ivf-candidate-batch-on --only latency-real100k-ivf-candidate-batch-off --only latency-real100k-ivf-candidate-batch-on --only storage-real100k-ivf --only pipeline-real100k-spire-candidate-batch-off --only pipeline-real100k-spire-candidate-batch-on --only storage-real100k-spire --manifest-output reviews/task-87/014-phase6-real100k-matrix/artifacts/real100k-run-manifest.json --results-output reviews/task-87/014-phase6-real100k-matrix/artifacts/real100k-results.jsonl --log-file reviews/task-87/014-phase6-real100k-matrix/artifacts/real100k-run.log`
- result: passed
- key cited lines:
  - `wrote reviews/task-87/014-phase6-real100k-matrix/artifacts/real100k-results.jsonl`

### `real100k-run-manifest.json`

- command: emitted by `real100k-run.log`.
- result: passed
- key cited status:
  - `completed=13 failed=0 skipped=28 dry_run=0 missing_artifacts=0 stale=0`

### `real100k-results.jsonl`

- command: emitted by `real100k-run.log`.
- result: written
- purpose: normalized suite result rows for the real100k slice.

### Truth Caches

- artifacts:
  - `run/truth-real100k-k10.json`
  - `run/truth-real100k-ivf-k10.json`
  - `run/truth-real100k-spire-k10.json`
- purpose: exact 100-query k=10 truth caches for HNSW, IVF, and SPIRE real100k prefixes.

### HNSW Run Logs

- artifacts:
  - `run/recall-real100k-hnsw-candidate-batch-off.log`
  - `run/recall-real100k-hnsw-candidate-batch-on.log`
  - `run/latency-real100k-hnsw-candidate-batch-off.log`
  - `run/latency-real100k-hnsw-candidate-batch-on.log`
  - `run/storage-real100k-hnsw.log`
- key cited lines:
  - recall@k: `0.8980` / `0.8980`
  - recall mean q-time: `61.99 ms` / `36.71 ms`
  - latency p50: `35.6 ms` / `35.5 ms`
  - latency p95: `43.8 ms` / `42.9 ms`
  - latency p99: `51.6 ms` / `50.1 ms`
  - storage: `total 1.7 GiB`, `indexes 132.4 MiB`

### IVF Run Logs

- artifacts:
  - `run/recall-real100k-ivf-candidate-batch-off.log`
  - `run/recall-real100k-ivf-candidate-batch-on.log`
  - `run/latency-real100k-ivf-candidate-batch-off.log`
  - `run/latency-real100k-ivf-candidate-batch-on.log`
  - `run/storage-real100k-ivf.log`
- key cited lines:
  - recall@k: `1.0000` / `1.0000`
  - recall mean q-time: `1093.79 ms` / `983.10 ms`
  - latency p50: `1064.2 ms` / `960.5 ms`
  - latency p95: `1114.9 ms` / `1018.0 ms`
  - latency p99: `1131.1 ms` / `1048.6 ms`
  - storage: `total 1.6 GiB`, `indexes 89.5 MiB`

### SPIRE Run Logs

- artifacts:
  - `run/pipeline-real100k-spire-candidate-batch-off.log`
  - `run/pipeline-real100k-spire-candidate-batch-on.log`
  - `run/storage-real100k-spire.log`
- key cited lines:
  - recall@k: `0.9100` / `0.9100`
  - pipeline p50: `414.768 ms` / `273.031 ms`
  - pipeline p95: `471.651 ms` / `298.031 ms`
  - pipeline p99: `495.905 ms` / `308.541 ms`
  - storage: `total 1.6 GiB`, `indexes 81.8 MiB`
