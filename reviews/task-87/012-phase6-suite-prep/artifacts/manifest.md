# Task 87 Packet 012 Artifact Manifest

- head SHA: `8d5fab4aebe35e094475588d1ba3bb89e19e813b`
- task bucket: `reviews/task-87/`
- packet path: `reviews/task-87/012-phase6-suite-prep/`
- timestamp: `2026-06-08T19:54:14Z`
- scope: Phase 6 aggregate measurement suite preparation, dry-run evidence, and setup execution
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
