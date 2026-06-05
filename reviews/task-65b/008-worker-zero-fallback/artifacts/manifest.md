# Task 65b Packet 008 Artifact Manifest

- head SHA: `be512b942a67d6a6708e471fc5af7e1318fd4ffe`
- task bucket: `reviews/task-65b/008-worker-zero-fallback`
- timestamp: `2026-06-05T03:02:45Z`
- lane: m5 local PG18
- fixture: DBpedia real10k and real100k
- profile: `ec_diskann`
- storage format: `pq_fastscan`
- rerank mode: default exact rerank through `ecaz bench recall`
- graph params: `graph_degree=32`, `build_list_size=100`, `alpha=1.2`
- worker-zero controls: suite load steps used `PGOPTIONS="-c max_parallel_maintenance_workers=0 -c max_parallel_workers=0"` plus table reloption `parallel_workers=0`
- index/table isolation: one index per table using prefixes `task65b_w0_real10k_r32_l100` and `task65b_w0_real100k_r32_l100`

## Commands

- install updated extension:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-65b/008-worker-zero-fallback/artifacts/install-ecaz-pg-test.log dev install ecaz-pg-test --pg 18`
- audit suite:
  `./target/debug/ecaz bench suite audit --config reviews/task-65b/008-worker-zero-fallback/suite.json > reviews/task-65b/008-worker-zero-fallback/artifacts/suite-audit.log 2>&1`
- dry-run suite:
  `./target/debug/ecaz bench suite run --config reviews/task-65b/008-worker-zero-fallback/suite.json --dry-run --manifest-output reviews/task-65b/008-worker-zero-fallback/artifacts/suite-dry-run-manifest.json > reviews/task-65b/008-worker-zero-fallback/artifacts/suite-dry-run.log 2>&1`
- full suite:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-65b/008-worker-zero-fallback/suite.json --manifest-output reviews/task-65b/008-worker-zero-fallback/artifacts/suite-manifest.json --results-output reviews/task-65b/008-worker-zero-fallback/artifacts/results.jsonl > reviews/task-65b/008-worker-zero-fallback/artifacts/suite-run.log 2>&1`
- graph digest refresh after rebuilding the CLI with the new renderer:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-65b/008-worker-zero-fallback/suite.json --only graph-real10k-w0 --only graph-real100k-w0 --manifest-output reviews/task-65b/008-worker-zero-fallback/artifacts/suite-graph-rerun-manifest.json > reviews/task-65b/008-worker-zero-fallback/artifacts/suite-graph-rerun.log 2>&1`

Note: `results.jsonl` is empty after the graph-only rerun because that rerun rewrote the configured results path without recall/storage steps. The durable sources for the completed run are `suite-run.log`, `suite-manifest.json`, and the per-step logs.

## Artifact Summary

- `suite.json`: checked-in `ecaz bench suite` config.
- `suite-audit.log`: `audit passed: 9 steps`.
- `suite-dry-run.log`, `suite-dry-run-manifest.json`: command expansion showing worker-zero `PGOPTIONS` and `parallel_workers=0`.
- `install-ecaz-pg-test.log`: installed backend sha256 `bd5ae1a3e4380c349b159e7bfcd62627f9fd838db91097c798dcd7df5b653f08`.
- `precheck-host.log`: PG18.3 on aarch64 macOS, `shared_buffers=128MB`, `maintenance_work_mem=64MB`, server defaults `max_parallel_workers=8`, `max_parallel_maintenance_workers=2`.
- `suite-run.log`, `suite-manifest.json`: completed 9-step worker-zero suite.
- `load-real10k-w0.log`: corpus sha256 `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`, queries sha256 `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`, completed prefix in `9.78s`.
- `recall-real10k-w0.log`: recall@10 `0.9965`, `0.9970`, `0.9975` for L64/L128/L200; mean q-time `0.61 ms`, `0.68 ms`, `0.78 ms`.
- `storage-real10k-w0.log`: DiskANN index `4.7 MiB`, `494.0 B` per row.
- `graph-real10k-w0.log`: reachable live fraction `1.000000`, neighbor refs `257058`, zero dead/invalid/self/duplicate/unresolvable refs, digests:
  - `live_node_tid_digest=b476ea9f9a43d92eff12389fab3a013060d0a1cfdc47665af859194b4764d1bd`
  - `adjacency_digest=af9fe980fb9d0f6149d4102a82d561af0fc7e9b2fde422f47acc5e1e3cf7f0b5`
  - `first_256_node_digest=da8ab263ef126cffc5e62ddd42969e86f58b75e860f8b87f1327649246e2a667`
- `load-real100k-w0.log`: corpus sha256 `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`, queries sha256 `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`, built index in `243.15s`, completed prefix in `411.92s`.
- `recall-real100k-w0.log`: recall@10 `0.9190`, `0.9640`, `0.9755` for L64/L128/L200; mean q-time `0.96 ms`, `1.11 ms`, `1.40 ms`.
- `storage-real100k-w0.log`: DiskANN index `46.1 MiB`, `483.1 B` per row.
- `graph-real100k-w0.log`: reachable live fraction `0.999890`, neighbor refs `3101446`, zero dead/invalid/self/duplicate/unresolvable refs, digests:
  - `live_node_tid_digest=5739d9a6040ccf6fe041e297d201a5a25537d18955398d9054c378926d81de53`
  - `adjacency_digest=683af2fb14938b475054f2d735d14e89a162947e93dba795d0077c5f492b5a12`
  - `first_256_node_digest=e332f9a4cba1318e4563adc9e2802d33ffefd161be3c76abf14eed503c31b4f7`
- `suite-graph-rerun.log`, `suite-graph-rerun-manifest.json`: graph digest refresh for both worker-zero indexes after the CLI graph output included digest rows.
- `truth-real10k-k10.json`, `truth-real100k-k10.json`: packet-local truth caches used by recall steps.

## Baseline Reconciliation

- Real10k recall matches the Slice A worker-zero values exactly at L64/L128/L200: `0.9965`, `0.9970`, `0.9975`.
- Real100k recall matches the Slice A worker-zero values exactly at L64/L128/L200: `0.9190`, `0.9640`, `0.9755`.
- Real100k build time is effectively unchanged from Slice A (`243.15s` here vs `243.29s` in packet `001`).
- This packet establishes digestable worker-zero fallback outputs at corpus scale. It does not prove byte equality against Task 65 head because the digest surface did not exist in the earlier Task 65 baseline packet.
