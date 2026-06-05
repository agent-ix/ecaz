# Artifact Manifest

- Head SHA: `7df607cc77f3692779f50471a3515fc8bb4288a6`
- Task bucket: `reviews/task-65b`
- Packet path: `reviews/task-65b/013-worker-batch-results`
- Timestamp: `2026-06-05T04:48:59Z`
- Lane: m5-local PG18
- Fixture: DBpedia real10k and real100k
- Profile: `ec_diskann`
- Storage format: `pq_fastscan`
- Rerank mode: not applicable to build-time measurement
- Graph params: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`
- Index/table isolation: one index per table through unique `task65b_sweep_*` prefixes
- Suite config: `reviews/task-65b/011-worker-batch-sweep/suite.json`

## Setup

### `artifacts/install-current-extension.log`

- Command: `./target/debug/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-65b/011-worker-batch-sweep/artifacts/install-current-extension.log`
- Result: passed.
- Key lines:
  - `Finished installing ecaz`
  - `backend artifact assertion passed`
  - `sha256=82d48b43b614fd52a7ada0faf72c77d83c7a72fa27298bfeab7d7b5c0b72f7b3`

### `artifacts/install-diskann-timing-helper.log`

- Command: `./target/debug/ecaz dev sql --pg 18 --db tqvector_bench --socket-dir /Users/peter/.pgrx --raw --sql "<CREATE OR REPLACE FUNCTION ec_diskann_last_build_timing...; ALTER EXTENSION...>"`
- Result: passed.
- Key lines:
  - `CREATE FUNCTION`
  - `ALTER EXTENSION`
  - helper count `1`

### `artifacts/precheck-host.log`

- Command: suite precheck step from `ecaz bench suite run`.
- Result: passed.
- Key lines:
  - PostgreSQL `18.3 (Homebrew)`
  - `max_parallel_workers = 8`
  - `max_parallel_maintenance_workers = 2`
  - extension `ecaz` version `0.1.1`

## Suite Run

### `artifacts/suite-run-host.log`

- Command:
  `./target/debug/ecaz bench suite run --config reviews/task-65b/011-worker-batch-sweep/suite.json --artifact-dir reviews/task-65b/013-worker-batch-results/artifacts --manifest-output reviews/task-65b/013-worker-batch-results/artifacts/suite-manifest-host.json --results-output reviews/task-65b/013-worker-batch-results/artifacts/results-host.jsonl --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-65b/013-worker-batch-results/artifacts/suite-run-host.log`
- Result: passed.
- Key lines:
  - `steps: completed 22, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0` in `artifacts/suite-report.md`
  - `wrote reviews/task-65b/013-worker-batch-results/artifacts/results-host.jsonl`

### `artifacts/suite-manifest-host.json`

- Manifest for the authoritative corrected suite run.
- Records all 22 steps as succeeded.

### `artifacts/results-host.jsonl`

- Normalized result rows parsed from the suite artifacts.
- Contains `115` rows.

Key build-time rows:

| Step | Build Time | Total Timing | Effective Workers | Batch | Reducer | Proposal | Epochs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| load-real10k-w1-b1 | 7.18 s | 7179 ms | 1 | 1 | 978 ms | 0 ms | 10000 |
| load-real10k-w2-b4 | 5.53 s | 5520 ms | 2 | 4 | 2021 ms | 171 ms | 2500 |
| load-real10k-w4-b8 | 5.38 s | 5372 ms | 4 | 8 | 3286 ms | 558 ms | 1250 |
| load-real10k-w4-b16 | 4.70 s | 4696 ms | 4 | 16 | 3105 ms | 726 ms | 625 |
| load-real10k-w8-b16 | 4.91 s | 4909 ms | 8 | 16 | 3552 ms | 546 ms | 625 |
| load-real10k-w8-b32 | 5.59 s | 5585 ms | 8 | 32 | 4432 ms | 603 ms | 313 |
| load-real100k-w4-b16 | 335.82 s | 335813 ms | 4 | 16 | 286136 ms | 36047 ms | 6250 |
| load-real100k-w8-b32 | 192.38 s | 192369 ms | 8 | 32 | 170504 ms | 15626 ms | 3125 |

Key recall rows at `list_size=200`:

| Step | Recall@10 | CI95 Low | CI95 High | Mean Query Time |
| --- | ---: | ---: | ---: | ---: |
| recall-real10k-w1-b1 | 0.9975 | 0.9942 | 0.9989 | 0.79 ms |
| recall-real10k-w4-b16 | 0.9975 | 0.9942 | 0.9989 | 0.79 ms |
| recall-real10k-w8-b32 | 0.9975 | 0.9942 | 0.9989 | 0.78 ms |
| recall-real100k-w4-b16 | 0.9750 | 0.9672 | 0.9810 | 1.40 ms |
| recall-real100k-w8-b32 | 0.9750 | 0.9672 | 0.9810 | 1.45 ms |

## Per-Step Artifacts

- `load-*.log`: corpus load, index build timing, and `ec_diskann_ambuild_timing` rows.
- `recall-*.log`: recall sweeps at `list_size` 64, 128, and 200.
- `graph-*.log`: DiskANN graph diagnostics and digest rows.
- `storage-*.log`: table and index storage summaries.
- `truth-real10k-k10.json`, `truth-real100k-k10.json`: suite-local ground-truth caches used by recall checks.

## Non-Authoritative Failed Invocation

- `artifacts/suite-run.log` and `artifacts/suite-manifest.json` are from an initial run that omitted global `--host /Users/peter/.pgrx --port 28818` for load steps.
- The run failed before producing a benchmark result at `load-real10k-w1-b1`.
- These files are retained only to explain the corrected invocation; do not use them as measurement evidence.
