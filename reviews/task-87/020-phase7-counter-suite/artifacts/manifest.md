# Task 87 Phase 7 Real10k Counter Suite Artifact Manifest

- Head SHA: `345a59659` plus packet-local PG18 extension install from the current branch.
- Task bucket: `reviews/task-87/`
- Packet path: `reviews/task-87/020-phase7-counter-suite/`
- Timestamp: `2026-06-08T15:59:47-07:00`
- Lane: local PG18 real10k counter suite.
- Fixture: DBPedia real10k surfaces reused from Task 87 Phase 6.
- Storage format: TurboQuant for the IVF and SPIRE touched-kernel surfaces.
- Rerank mode: IVF heap_f32 rerank width 25; SPIRE rerank width 25.
- Isolation: reuses the isolated one-index-per-table real10k surfaces from packet 012.

## Suite Config

### `phase7-real10k-counter-suite.json`

- Checked-in `ecaz bench suite` config for real10k Phase 7 counter capture.
- Includes IVF off/on recall, latency, and storage.
- Includes SPIRE off/on pipeline and storage.
- Includes HNSW candidate-batch-on latency for the batch-width decision probe.

## Setup Artifacts

### `artifacts/install-ecaz-pg-test.log`

- Command: `target/debug/ecaz --log-file reviews/task-87/020-phase7-counter-suite/artifacts/install-ecaz-pg-test.log dev install ecaz-pg-test --pg 18`
- Key result: installed backend assertion passed.
- Installed backend SHA256: `23b39c72fcfd2071c07db950363f9d30e93940f0d48053c67172568e185261e4`.

### `artifacts/counter-function-check.log`

- Command: `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --sql "SELECT to_regprocedure('ec_task87_candidate_batch_scoring_reset()') AS reset_fn, to_regprocedure('ec_task87_candidate_batch_scoring_snapshot()') AS snapshot_fn;"`
- Key result: both functions were absent before registration.

### `artifacts/register-task87-counter-functions.sql`

- Packet-local SQL used to register the two new counter functions in the existing same-version `postgres` database.

### `artifacts/register-task87-counter-functions.log`

- Command: `target/debug/ecaz dev sql --pg 18 --db postgres --socket-dir /home/peter/.pgrx --port 28818 --raw --file reviews/task-87/020-phase7-counter-suite/artifacts/register-task87-counter-functions.sql`
- Key result: both `CREATE FUNCTION` statements succeeded.

### `artifacts/counter-function-check-after-register.log`

- Key result: `ec_task87_candidate_batch_scoring_reset()` and `ec_task87_candidate_batch_scoring_snapshot()` are registered.

## Suite Artifacts

### `artifacts/suite-audit.log`

- Command: `target/debug/ecaz bench suite audit --config reviews/task-87/020-phase7-counter-suite/phase7-real10k-counter-suite.json`
- Key result: audit passed, 10 steps.

### `artifacts/suite-dry-run.log`

- Command: `target/debug/ecaz bench suite run --config reviews/task-87/020-phase7-counter-suite/phase7-real10k-counter-suite.json --dry-run --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-87/020-phase7-counter-suite/artifacts/suite-dry-run-manifest.json --results-output reviews/task-87/020-phase7-counter-suite/artifacts/suite-dry-run-results.jsonl --log-file reviews/task-87/020-phase7-counter-suite/artifacts/suite-dry-run.log`
- Key result: expanded all 10 steps; counter capture flag is present on IVF latency, SPIRE pipeline, and HNSW latency.

### `artifacts/real10k-run.log`

- Command: `target/debug/ecaz bench suite run --config reviews/task-87/020-phase7-counter-suite/phase7-real10k-counter-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-87/020-phase7-counter-suite/artifacts/real10k-run-manifest.json --results-output reviews/task-87/020-phase7-counter-suite/artifacts/real10k-results.jsonl --log-file reviews/task-87/020-phase7-counter-suite/artifacts/real10k-run.log`
- Key result: suite completed all 10 steps.

### `artifacts/real10k-run-manifest.json`

- Suite run manifest for the real10k counter run.

### `artifacts/real10k-results.jsonl`

- Parsed result rows from suite artifacts.

## Key Per-Step Logs

### `artifacts/run/recall-real10k-ivf-candidate-batch-off.log`

- Key result: recall@10 `1.0000`, mean q-time `21.43 ms`.

### `artifacts/run/recall-real10k-ivf-candidate-batch-on.log`

- Key result: recall@10 `1.0000`, mean q-time `16.89 ms`.

### `artifacts/run/latency-real10k-ivf-candidate-batch-off.log`

- Key result: p50 `19.7 ms`, p95 `20.9 ms`, p99 `23.5 ms`; IVF counters zero.

### `artifacts/run/latency-real10k-ivf-candidate-batch-on.log`

- Key result: p50 `16.7 ms`, p95 `18.7 ms`, p99 `23.2 ms`.
- Counter result: IVF `flushes=8000`, `candidates=2000000`, `elapsed_ms=2302.509555`, `lut32_flushes=7800`, `lut32_candidates=1996800`.

### `artifacts/run/pipeline-real10k-spire-candidate-batch-off.log`

- Key result: recall@10 `1.0000`; p50 `19.091 ms`, p95 `22.515 ms`, p99 `24.009 ms`; SPIRE counters zero.

### `artifacts/run/pipeline-real10k-spire-candidate-batch-on.log`

- Key result: recall@10 `1.0000`; p50 `16.951 ms`, p95 `18.087 ms`, p99 `20.206 ms`.
- Counter result: SPIRE `flushes=157548`, `candidates=1551640`, `elapsed_ms=2169.038804`, `lut32_flushes=0`, `lut32_candidates=0`.

### `artifacts/run/latency-real10k-hnsw-candidate-batch-on.log`

- Key result: p50 `5.19 ms`, p95 `11.2 ms`, p99 `36.4 ms`.
- Counter result: all Task 87 counter surfaces zero.

### `artifacts/run/storage-real10k-ivf.log`

- Key result: total `168.2 MiB`, indexes `9.4 MiB`.

### `artifacts/run/storage-real10k-spire.log`

- Key result: total `167.0 MiB`, indexes `8.2 MiB`.
