# Task 87 Packet 013 Artifact Manifest

- head SHA: `ed5338d07c8d3b9e9071c6ab2281119373b846b54`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/013-phase6-real50k-matrix/`
- timestamp: `2026-06-08T22:46:00Z`
- scope: Phase 6 real50k measurement slice
- lane / fixture / storage format / rerank mode: local PG18; real50k; HNSW TurboQuant existing surface; IVF existing RaBitQ surface; SPIRE suite-owned TurboQuant surface; IVF/SPIRE rerank width 25
- isolated one-index-per-table vs shared-table surfaces: reused one-AM HNSW and IVF tables; suite-owned one-AM SPIRE table; each table has one ANN index plus btree primary key

## Suite Config

- source config: `reviews/task-87/012-phase6-suite-prep/phase6-suite.json`
- suite update before this run:
  - real50k/real100k HNSW and IVF recall steps now set `queries_limit = 100`.
  - real50k/real100k IVF recall steps use AM-specific truth caches.
  - SPIRE pipeline steps already used AM-specific truth caches and `queries_limit = 100`.
- audit result after suite update:
  - command: `target/debug/ecaz bench suite audit --config reviews/task-87/012-phase6-suite-prep/phase6-suite.json`
  - result: `[suite:task87-phase6-candidate-batch-matrix] audit passed: 41 steps`

## Query-Set Check

- command: read-only PG18 hash query over the real50k HNSW/IVF/SPIRE query tables.
- result:
  - `hnsw50|1000|f100c216c348f3582baada81cbeb981d`
  - `ivf50|435|313bf69e2c56cd33c9175244ae209f57`
  - `spire50|1000|f100c216c348f3582baada81cbeb981d`
- consequence: IVF requires `truth-real50k-ivf-k10.json`; SPIRE uses
  `truth-real50k-spire-k10.json` because `spire-pipeline` requires a
  pre-existing truth cache.

## Artifacts

### `real50k-run.log`

- command: `target/debug/ecaz bench suite run --config reviews/task-87/012-phase6-suite-prep/phase6-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --only recall-real50k-hnsw-candidate-batch-off --only recall-real50k-hnsw-candidate-batch-on --only latency-real50k-hnsw-candidate-batch-off --only latency-real50k-hnsw-candidate-batch-on --only storage-real50k-hnsw --only recall-real50k-ivf-candidate-batch-off --only recall-real50k-ivf-candidate-batch-on --only latency-real50k-ivf-candidate-batch-off --only latency-real50k-ivf-candidate-batch-on --only storage-real50k-ivf --only pipeline-real50k-spire-candidate-batch-off --only pipeline-real50k-spire-candidate-batch-on --only storage-real50k-spire --manifest-output reviews/task-87/012-phase6-suite-prep/artifacts/real50k-run-manifest.json --results-output reviews/task-87/012-phase6-suite-prep/artifacts/real50k-results.jsonl --log-file reviews/task-87/012-phase6-suite-prep/artifacts/real50k-run.log`
- result: partial
- key cited lines:
  - HNSW and IVF real50k recall, latency, and storage cells completed.
  - SPIRE first pipeline cell failed because `truth-real50k-spire-k10.json`
    did not exist yet.

### `real50k-run-manifest.json`

- command: emitted by `real50k-run.log`.
- result: written
- key cited status:
  - 10 completed cells, 1 failed SPIRE pipeline cell before truth-cache generation.

### `run/truth-real50k-k10.json`

- command: emitted by `recall-real50k-hnsw-candidate-batch-off`.
- result: written
- purpose: exact 100-query k=10 truth cache for the HNSW real50k prefix.

### `run/truth-real50k-ivf-k10.json`

- command: emitted by `recall-real50k-ivf-candidate-batch-off`.
- result: written
- purpose: exact 100-query k=10 truth cache for the distinct IVF real50k query table.

### `run/truth-real50k-spire-generate.log`

- command: `target/debug/ecaz bench recall --database postgres --host /home/peter/.pgrx --port 28818 --prefix task87_phase6_real50k_spire --profile ec_spire --k 10 --sweep 24 --rerank-width 25 --queries-limit 100 --bits 4 --seed 42 --force-index --session-guc ec_spire.candidate_batch_scoring=off --truth-cache-file reviews/task-87/012-phase6-suite-prep/artifacts/run/truth-real50k-spire-k10.json --log-output reviews/task-87/012-phase6-suite-prep/artifacts/run/truth-real50k-spire-generate.log`
- result: passed
- key cited lines:
  - `ground truth in 8.98s`
  - `wrote ground truth cache reviews/task-87/012-phase6-suite-prep/artifacts/run/truth-real50k-spire-k10.json`
  - `recall@k 0.9690`
  - `mean q-time 226.73 ms`

### `run/truth-real50k-spire-k10.json`

- command: emitted by `run/truth-real50k-spire-generate.log`.
- result: written
- purpose: exact 100-query k=10 truth cache for the SPIRE real50k prefix.

### `real50k-spire-rerun.log`

- command: `target/debug/ecaz bench suite run --config reviews/task-87/012-phase6-suite-prep/phase6-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --only pipeline-real50k-spire-candidate-batch-off --only pipeline-real50k-spire-candidate-batch-on --only storage-real50k-spire --manifest-output reviews/task-87/012-phase6-suite-prep/artifacts/real50k-spire-rerun-manifest.json --results-output reviews/task-87/012-phase6-suite-prep/artifacts/real50k-spire-rerun-results.jsonl --log-file reviews/task-87/012-phase6-suite-prep/artifacts/real50k-spire-rerun.log`
- result: passed
- key cited lines:
  - `pipeline-real50k-spire-candidate-batch-off`
  - `pipeline-real50k-spire-candidate-batch-on`
  - `storage-real50k-spire`

### `real50k-spire-rerun-manifest.json`

- command: emitted by `real50k-spire-rerun.log`.
- result: passed
- key cited status:
  - `completed=3 failed=0 skipped=38 dry_run=0 missing_artifacts=0 stale=0`

### `real50k-spire-rerun-results.jsonl`

- command: emitted by `real50k-spire-rerun.log`.
- result: written
- key cited rows:
  - off: `latency_p50=224.610 ms`, `latency_p95=255.674 ms`,
    `latency_p99=266.182 ms`, `recall@k=0.9690`
  - on: `latency_p50=160.449 ms`, `latency_p95=180.921 ms`,
    `latency_p99=186.580 ms`, `recall@k=0.9690`
  - storage: `total=834.3 MiB`, `indexes=40.5 MiB`

### HNSW Run Logs

- artifacts:
  - `run/recall-real50k-hnsw-candidate-batch-off.log`
  - `run/recall-real50k-hnsw-candidate-batch-on.log`
  - `run/latency-real50k-hnsw-candidate-batch-off.log`
  - `run/latency-real50k-hnsw-candidate-batch-on.log`
  - `run/storage-real50k-hnsw.log`
- key cited lines:
  - recall@k: `0.9180` / `0.9180`
  - recall mean q-time: `43.66 ms` / `32.47 ms`
  - latency p50: `32.4 ms` / `31.3 ms`
  - latency p95: `42.1 ms` / `37.9 ms`
  - latency p99: `58.5 ms` / `41.9 ms`
  - storage: `total 860.0 MiB`, `indexes 66.2 MiB`

### IVF Run Logs

- artifacts:
  - `run/recall-real50k-ivf-candidate-batch-off.log`
  - `run/recall-real50k-ivf-candidate-batch-on.log`
  - `run/latency-real50k-ivf-candidate-batch-off.log`
  - `run/latency-real50k-ivf-candidate-batch-on.log`
  - `run/storage-real50k-ivf.log`
- key cited lines:
  - recall@k: `0.9300` / `0.9300`
  - recall mean q-time: `266.77 ms` / `264.18 ms`
  - latency p50: `264.0 ms` / `264.3 ms`
  - latency p95: `289.7 ms` / `292.9 ms`
  - latency p99: `311.6 ms` / `308.5 ms`
  - storage: `total 840.9 MiB`, `indexes 47.1 MiB`

### SPIRE Run Logs

- artifacts:
  - `run/pipeline-real50k-spire-candidate-batch-off.log`
  - `run/pipeline-real50k-spire-candidate-batch-on.log`
  - `run/storage-real50k-spire.log`
- key cited lines:
  - recall@k: `0.9690` / `0.9690`
  - pipeline p50: `224.610 ms` / `160.449 ms`
  - pipeline p95: `255.674 ms` / `180.921 ms`
  - pipeline p99: `266.182 ms` / `186.580 ms`
  - storage: `total 834.3 MiB`, `indexes 40.5 MiB`
