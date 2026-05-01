# Task 29e Artifact Manifest

Packet: `review/11110-task29e-rerank-borrowed-simd`  
Timestamp: `2026-05-01T08:00:35-07:00`  
Primary head SHA: `009d433c` (`Reuse DiskANN SIMD inner product for heap rerank`)  
Environment: local PG18 pgrx server, socket `/home/peter/.pgrx`, port `28818`,
database `task29_diskann_baseline`.

## Release Install / Restart

Command:

```text
cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features pg18
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl restart -D /home/peter/.pgrx/data-18
```

The install/restart command output was observed in-session. The benchmark logs
below are the packet-local source of truth for measurement claims.

## Isolated `ec_diskann` Recall

Artifacts:

- `recall-sweep-rerank-borrowed-simd-isolated-cli.log`
- `recall-sweep-rerank-borrowed-simd-isolated-table.log`

Lane / fixture / storage / rerank:

- `ec_diskann`, real-10k, `k=10`, list-size sweep `64,128,200,400,800`
- storage format: `pq_fastscan` with binary sidecar
- rerank mode: exact heap-source rerank
- isolated one-index-per-table surface for `ec_diskann`; HNSW reference indexes
  were dropped before this run.

Command:

```text
target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 bench recall --prefix task29c_phase_profile --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --force-index --truth-cache-file review/11109-task29d-final-readiness/artifacts/truth-v1-rows10000-queries200-dim1536-k10-4473cd157aa35fa6.json --log-file review/11110-task29e-rerank-borrowed-simd/artifacts/recall-sweep-rerank-borrowed-simd-isolated-cli.log --log-output review/11110-task29e-rerank-borrowed-simd/artifacts/recall-sweep-rerank-borrowed-simd-isolated-table.log
```

Key cited rows:

- L=64: recall `0.9965`, NDCG `0.9999`, mean q-time `7.90 ms`
- L=128: recall `0.9965`, NDCG `0.9999`, mean q-time `7.83 ms`
- L=200: recall `0.9970`, NDCG `0.9999`, mean q-time `8.19 ms`
- L=400: recall `0.9970`, NDCG `0.9999`, mean q-time `8.62 ms`
- L=800: recall `0.9975`, NDCG `0.9999`, mean q-time `9.76 ms`

## Isolated `ec_diskann` Latency

Artifacts:

- `latency-sweep-rerank-borrowed-simd-isolated-cli.log`
- `latency-sweep-rerank-borrowed-simd-isolated-table.log`

Lane / fixture / storage / rerank:

- `ec_diskann`, real-10k, `k=10`, list-size sweep `64,128,200,400,800`,
  500 iterations, concurrency 1
- storage format: `pq_fastscan` with binary sidecar
- rerank mode: exact heap-source rerank
- isolated one-index-per-table surface for `ec_diskann`; HNSW reference indexes
  were dropped before this run.

Command:

```text
target/release/ecaz --database task29_diskann_baseline --host /home/peter/.pgrx --port 28818 bench latency --prefix task29c_phase_profile --profile ec_diskann --k 10 --sweep 64,128,200,400,800 --iterations 500 --concurrency 1 --force-index --sample-backend-memory --log-file review/11110-task29e-rerank-borrowed-simd/artifacts/latency-sweep-rerank-borrowed-simd-isolated-cli.log --log-output review/11110-task29e-rerank-borrowed-simd/artifacts/latency-sweep-rerank-borrowed-simd-isolated-table.log
```

Key cited rows:

- L=64: mean `7.70 ms`, p50 `7.61 ms`, p95 `8.44 ms`, p99 `8.89 ms`, HWM `61660 kB`
- L=128: mean `7.76 ms`, p50 `7.69 ms`, p95 `8.43 ms`, p99 `9.45 ms`, HWM `61664 kB`
- L=200: mean `8.10 ms`, p50 `8.05 ms`, p95 `8.86 ms`, p99 `10.1 ms`, HWM `61820 kB`
- L=400: mean `8.60 ms`, p50 `8.51 ms`, p95 `9.40 ms`, p99 `10.1 ms`, HWM `62788 kB`
- L=800: mean `9.33 ms`, p50 `9.23 ms`, p95 `10.4 ms`, p99 `11.0 ms`, HWM `63044 kB`

## Rejected / Audit-Only Artifacts

The following artifacts are retained so the packet records why the experiments
were not kept. They are not landing claims.

- `latency-sweep-rerank-borrowed-simd-cli.log`
- `latency-sweep-rerank-borrowed-simd-table.log`
- `recall-sweep-rerank-borrowed-simd-cli.log`
- `recall-sweep-rerank-borrowed-simd-table.log`

These were run before dropping HNSW reference indexes. The flat recall
`0.9595` at every L exposed planner ambiguity and invalidates those rows.

- `latency-sweep-rerank-borrowed-simd-neighbor-consume-isolated-cli.log`
- `latency-sweep-rerank-borrowed-simd-neighbor-consume-isolated-table.log`

Uncommitted scan neighbor-list consume experiment, isolated surface. Rejected
because higher-L latency regressed:

- L=400 mean `9.18 ms`, p99 `13.9 ms`
- L=800 mean `9.83 ms`, p99 `14.2 ms`

- `create-index-build-scratch-release.log`

Uncommitted build epoch-mark scratch experiment. Rejected because real-10k
rebuild regressed:

- total build `15155 ms`
- build/persist `13506 ms`
- core graph `13282 ms`

## Validation

Code validation before commit:

```text
cargo check --all-targets --no-default-features --features pg18
cargo test source_inner_product_dispatch_matches_scalar --no-default-features --features pg18
cargo pgrx test pg18 test_ec_diskann_sql_ordered_index_scan_executes
cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings
git diff --check
```
