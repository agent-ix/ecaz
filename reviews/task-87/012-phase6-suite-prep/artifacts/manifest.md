# Task 87 Packet 012 Artifact Manifest

- head SHA: `bbe3ef9e8b376c632267c0b39089d3b4ac483373`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/012-phase6-suite-prep/`
- timestamp: `2026-06-08T22:18:00Z`
- scope: Phase 6 aggregate measurement suite preparation, dry-run evidence, setup execution, and real10k measurement slice
- lane / fixture / storage format / rerank mode: local PG18; real10k/real50k/real100k; TurboQuant 4-bit for 10k suite-owned HNSW/IVF/SPIRE, 50k SPIRE, 100k IVF/SPIRE; 50k IVF uses existing RaBitQ surface; IVF/SPIRE rerank width 25 where applicable
- isolated one-index-per-table vs shared-table surfaces: suite-owned 10k HNSW/IVF/SPIRE and 50k SPIRE surfaces are isolated; 50k/100k HNSW/IVF and 100k SPIRE reuse existing one-AM real-corpus tables with a single AM index plus btree primary key

## Artifacts

### `phase6-suite.json`

- command target: `ecaz bench suite run --config reviews/task-87/012-phase6-suite-prep/phase6-suite.json`
- purpose: checked-in suite config for Task 87 Phase 6 off/on matrix.
- key shape:
  - 41 total steps.
  - setup/precheck raw steps.
  - HNSW recall/latency off/on cells via `ec_hnsw.candidate_batch_scoring`.
  - IVF recall/latency off/on cells via `ec_ivf.scratch_soa_batch_decode`.
  - SPIRE pipeline recall/latency off/on cells via `ec_spire.candidate_batch_scoring`.
  - SPIRE pipeline cells use `queries_limit = 100` and AM-specific truth cache
    files because the SPIRE prefixes are separately loaded query tables.
  - storage steps per isolated AM/corpus surface.

### `prepare-isolated-surfaces.sql`

- command: invoked by suite raw step `prepare-isolated-surfaces`.
- result: executed by `setup-run.log`.
- purpose: idempotently creates suite-owned real10k HNSW/IVF/SPIRE surfaces and the missing real50k SPIRE surface.

### `cargo-build-ecaz-cli.log`

- command: `cargo build -p ecaz-cli`
- result: passed
- key cited lines:
  - `warning: ecaz-cli (bin "ecaz") generated 1 warning`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 0.18s`

### `corpus-list-pg18.log`

- command: `target/debug/ecaz corpus list --database postgres --host /home/peter/.pgrx --port 28818`
- result: passed
- key cited lines:
  - `task30_spire_real10k_tq`
  - `task67_local_fullq_50k_hnsw`
  - `task67_local_fullq_100k_hnsw`
  - `task74_intel_spire_highrecall_tg128_b0`

### `catalog-prefixes.log`

- command: read-only PG18 catalog query over loaded real-corpus prefixes.
- result: passed
- key cited lines:
  - `task28_ivf_qcmp10k_turboquant`
  - `task28_ivf_tq100k_n64w25`
  - `task30_spire_real10k_tq`
  - `task67_local_fullq_50k_hnsw`
  - `task67_local_fullq_100k_hnsw`
  - `task74_intel_spire_highrecall_tg128_b0`

### `selected-index-reloptions.log`

- command: read-only PG18 reloption query for source HNSW/IVF/SPIRE indexes.
- result: passed
- key cited lines:
  - `storage_format=turboquant`
  - `m=16,ef_construction=128`
  - `nlists=64,nprobe=64,training_sample_rows=2000,storage_format=turboquant,rerank=heap_f32,rerank_width=25`
  - `nlists=128,recursive_fanout=8,nprobe=24,rerank_width=25,storage_format=turboquant`

### `suite-audit.log`

- command: `target/debug/ecaz bench suite audit --config reviews/task-87/012-phase6-suite-prep/phase6-suite.json`
- result: passed
- key cited lines:
  - `[suite:task87-phase6-candidate-batch-matrix] audit passed: 41 steps`

### `suite-dry-run.log`

- command: `target/debug/ecaz bench suite run --config reviews/task-87/012-phase6-suite-prep/phase6-suite.json --dry-run --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-87/012-phase6-suite-prep/artifacts/suite-dry-run-manifest.json --results-output reviews/task-87/012-phase6-suite-prep/artifacts/suite-dry-run-results.jsonl`
- result: passed
- key cited lines:
  - `wrote reviews/task-87/012-phase6-suite-prep/artifacts/suite-dry-run-manifest.json`
  - `--session-guc ec_hnsw.candidate_batch_scoring=off`
  - `--session-guc ec_hnsw.candidate_batch_scoring=on`
  - `--ivf-scratch-soa-batch-decode`
  - `bench spire-pipeline --prefix task87_phase6_real10k_spire --queries-limit 100`
  - `--session-guc ec_spire.candidate_batch_scoring=off`
  - `--session-guc ec_spire.candidate_batch_scoring=on`

### `suite-dry-run-manifest.json`

- command: emitted by the dry-run command above.
- result: written
- purpose: normalized suite manifest for reviewer inspection before the long run.

### `setup-run.log`

- command: `target/debug/ecaz bench suite run --config reviews/task-87/012-phase6-suite-prep/phase6-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --only precheck-host --only prepare-isolated-surfaces --manifest-output reviews/task-87/012-phase6-suite-prep/artifacts/setup-run-manifest.json --results-output reviews/task-87/012-phase6-suite-prep/artifacts/setup-run-results.jsonl`
- result: passed
- key cited lines:
  - `precheck-host -> --database postgres --host /home/peter/.pgrx --port 28818 dev sql`
  - `prepare-isolated-surfaces -> --database postgres --host /home/peter/.pgrx --port 28818 dev sql`
  - `wrote reviews/task-87/012-phase6-suite-prep/artifacts/setup-run-results.jsonl`

### `run/precheck-host.log`

- command: emitted by setup-run `precheck-host` step.
- result: passed
- key cited lines:
  - `PostgreSQL 18.3 on x86_64-pc-linux-gnu`
  - `shared_buffers | work_mem | maintenance_work_mem | effective_cache_size`

### `run/prepare-isolated-surfaces.log`

- command: emitted by setup-run `prepare-isolated-surfaces` step.
- result: passed
- key cited lines:
  - `relation "task87_phase6_real10k_hnsw_m16_idx" already exists, skipping`
  - `relation "task87_phase6_real10k_ivf_tq_idx" already exists, skipping`
  - `relation "task87_phase6_real10k_spire_tq_idx" already exists, skipping`
  - `ec_spire_ambuild_timing index=task87_phase6_real50k_spire_tq_idx phase=complete`
  - `total_ms=306144`

### `setup-run-manifest.json`

- command: emitted by setup-run command above.
- result: written
- purpose: normalized manifest for the setup-only execution.

### `setup-run-results.jsonl`

- command: emitted by setup-run command above.
- result: written empty file because setup/raw steps do not produce normalized benchmark rows.

### `refresh-spire-pipeline-functions.sql`

- command: generated from `/home/peter/.pgrx/18.3/pgrx-install/share/postgresql/extension/ecaz--0.1.1.sql`.
- purpose: narrow local PG18 refresh for the C wrappers used by
  `ecaz bench spire-pipeline`; substitutes `$libdir/ecaz` for extension-only
  `MODULE_PATHNAME`.
- key shape:
  - 15 wrapper declarations.
  - includes `ec_spire_remote_search_endpoint_identity`,
    `ec_spire_index_scan_routing_snapshot`,
    `ec_spire_index_scan_pipeline_snapshot`, and the other SPIRE pipeline
    diagnostics queried by the CLI.

### `refresh-spire-pipeline-functions.log`

- command: `/home/peter/.pgrx/18.3/pgrx-install/bin/psql -h /home/peter/.pgrx -p 28818 -d postgres -f reviews/task-87/012-phase6-suite-prep/artifacts/refresh-spire-pipeline-functions.sql`
- result: passed
- key cited lines:
  - 15 `CREATE FUNCTION` statements completed.

### `real10k-run.log`

- command: `target/debug/ecaz bench suite run --config reviews/task-87/012-phase6-suite-prep/phase6-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --resume-from reviews/task-87/012-phase6-suite-prep/artifacts/setup-run-manifest.json --only recall-real10k-hnsw-candidate-batch-off --only recall-real10k-hnsw-candidate-batch-on --only latency-real10k-hnsw-candidate-batch-off --only latency-real10k-hnsw-candidate-batch-on --only storage-real10k-hnsw --only recall-real10k-ivf-candidate-batch-off --only recall-real10k-ivf-candidate-batch-on --only latency-real10k-ivf-candidate-batch-off --only latency-real10k-ivf-candidate-batch-on --only storage-real10k-ivf --only pipeline-real10k-spire-candidate-batch-off --only pipeline-real10k-spire-candidate-batch-on --only storage-real10k-spire --manifest-output reviews/task-87/012-phase6-suite-prep/artifacts/real10k-run-manifest.json --results-output reviews/task-87/012-phase6-suite-prep/artifacts/real10k-results.jsonl --log-file reviews/task-87/012-phase6-suite-prep/artifacts/real10k-run.log`
- result: partial
- key cited lines:
  - HNSW and IVF real10k recall, latency, and storage cells completed.
  - SPIRE first pipeline cell initially failed because the local PG18 extension
    catalog was missing `ec_spire_remote_search_endpoint_identity(oid)`.

### `real10k-run-manifest.json`

- command: emitted by the partial real10k run above.
- result: written
- key cited status:
  - 10 completed cells, 1 failed SPIRE pipeline cell before wrapper refresh.

### `run/truth-real10k-k10.json`

- command: emitted by `recall-real10k-hnsw-candidate-batch-off`.
- result: written
- purpose: exact 100-query k=10 truth cache shared by the suite-owned HNSW and
  IVF real10k prefixes.

### `run/truth-real10k-spire-generate.log`

- command: `target/debug/ecaz bench recall --database postgres --host /home/peter/.pgrx --port 28818 --prefix task87_phase6_real10k_spire --profile ec_spire --k 10 --sweep 24 --rerank-width 25 --queries-limit 100 --bits 4 --seed 42 --force-index --session-guc ec_spire.candidate_batch_scoring=off --truth-cache-file reviews/task-87/012-phase6-suite-prep/artifacts/run/truth-real10k-spire-k10.json --log-output reviews/task-87/012-phase6-suite-prep/artifacts/run/truth-real10k-spire-generate.log`
- result: passed
- key cited lines:
  - `ground truth in 1.74s`
  - `wrote ground truth cache reviews/task-87/012-phase6-suite-prep/artifacts/run/truth-real10k-spire-k10.json`
  - `recall@k 1.0000`
  - `mean q-time 163.20 ms`

### `run/truth-real10k-spire-k10.json`

- command: emitted by `run/truth-real10k-spire-generate.log`.
- result: written
- purpose: exact 100-query k=10 truth cache for the separately loaded
  suite-owned SPIRE real10k prefix.

### `real10k-spire-rerun.log`

- command: `target/debug/ecaz bench suite run --config reviews/task-87/012-phase6-suite-prep/phase6-suite.json --database postgres --host /home/peter/.pgrx --port 28818 --only pipeline-real10k-spire-candidate-batch-off --only pipeline-real10k-spire-candidate-batch-on --only storage-real10k-spire --manifest-output reviews/task-87/012-phase6-suite-prep/artifacts/real10k-spire-rerun-manifest.json --results-output reviews/task-87/012-phase6-suite-prep/artifacts/real10k-spire-rerun-results.jsonl --log-file reviews/task-87/012-phase6-suite-prep/artifacts/real10k-spire-rerun.log`
- result: passed
- key cited lines:
  - `pipeline-real10k-spire-candidate-batch-off`
  - `pipeline-real10k-spire-candidate-batch-on`
  - `storage-real10k-spire`

### `real10k-spire-rerun-manifest.json`

- command: emitted by the SPIRE rerun command above.
- result: passed
- key cited status:
  - `completed=3 failed=0 skipped=38 dry_run=0 missing_artifacts=0 stale=0`

### `real10k-spire-rerun-results.jsonl`

- command: emitted by the SPIRE rerun command above.
- result: written
- key cited rows:
  - off: `latency_p50=168.137 ms`, `latency_p95=187.051 ms`,
    `recall@k=1.0000`
  - on: `latency_p50=106.142 ms`, `latency_p95=122.507 ms`,
    `recall@k=1.0000`
  - storage: `total=167.0 MiB`, `indexes=8.2 MiB`

### Real10k HNSW/IVF Run Logs

- commands: emitted by `real10k-run.log`.
- result: passed for HNSW and IVF cells.
- artifacts:
  - `run/recall-real10k-hnsw-candidate-batch-off.log`
  - `run/recall-real10k-hnsw-candidate-batch-on.log`
  - `run/latency-real10k-hnsw-candidate-batch-off.log`
  - `run/latency-real10k-hnsw-candidate-batch-on.log`
  - `run/storage-real10k-hnsw.log`
  - `run/recall-real10k-ivf-candidate-batch-off.log`
  - `run/recall-real10k-ivf-candidate-batch-on.log`
  - `run/latency-real10k-ivf-candidate-batch-off.log`
  - `run/latency-real10k-ivf-candidate-batch-on.log`
  - `run/storage-real10k-ivf.log`
- key cited lines:
  - HNSW off/on recall@k: `0.6550` / `0.6550`
  - HNSW off/on latency p50: `32.6 ms` / `31.6 ms`
  - HNSW storage: `total 171.8 MiB`, `indexes 13.0 MiB`
  - IVF off/on recall@k: `1.0000` / `1.0000`
  - IVF off/on latency p50: `119.6 ms` / `117.4 ms`
  - IVF storage: `total 168.2 MiB`, `indexes 9.4 MiB`

### Real10k SPIRE Run Logs

- commands: emitted by `real10k-spire-rerun.log`.
- result: passed for SPIRE cells.
- artifacts:
  - `run/pipeline-real10k-spire-candidate-batch-off.log`
  - `run/pipeline-real10k-spire-candidate-batch-on.log`
  - `run/storage-real10k-spire.log`
- key cited lines:
  - SPIRE off/on recall@k: `1.0000` / `1.0000`
  - SPIRE off/on pipeline p50: `168.137 ms` / `106.142 ms`
  - SPIRE off/on pipeline p95: `187.051 ms` / `122.507 ms`
  - SPIRE storage: `total 167.0 MiB`, `indexes 8.2 MiB`
