# Artifact Manifest

Packet: `reviews/task-111h/028-rerank-suite-1m-v7-shared`

Task bucket: `reviews/task-111h`

Head SHA before packet commit: `9f8432220c65b8d0d590d29899e5cb6e3874f44f`

Created: `2026-06-20`

Suite name: `task111h-1m-rerank-format-width-v7-shared`

## Environment

- PostgreSQL: `PostgreSQL 18.3 on x86_64-pc-linux-gnu`
- Socket dir: `/home/peter/.pgrx`
- Port: `28818`
- Database: `task111h_rerank_1m_v7`
- Shared buffers: `128MB`
- Work mem: `4MB`
- Maintenance work mem: `64MB`
- Effective cache size: `4GB`

The host precheck is in `artifacts/suite/precheck-host.log`.

## Corpus And Query Inputs

- Dataset: `dbpedia-openai3-large-1536-1m`
- Prepared profile: `ec_real_ann_benchmarks_anchor`
- Corpus rows: `990000`
- Query rows: `10000`
- Chunk rows: `25000`
- Staged manifest: `data/benchmark-profile-inputs/dbpedia-openai3-1m-staged/ec_real_ann_benchmarks_anchor_manifest.json`
- Staged manifest sha256: `546d2f3d8158efe3860e9b4074a5f185a0f8849fdfeb7d16768ec5c7a1c2fca4`

The recall truth cache was generated at `artifacts/suite/truth-1m-k10.json`. It is intentionally not committed because truth caches are regenerable packet input under the repository packet rules.

## Suite Configuration

Suite config: `artifacts/task111h-1m-rerank-format-width-v7-shared-suite.json`

Suite config sha256: `2d12a65841197c84c846af77389cecccb9da38f032d84087afe1e6d04a729d2f`

Shared-table shape:

- Prefix: `task111h028_1m_shared`
- One active IVF index per cell
- Drop-before and drop-after steps around every cell
- `storage_format=coarse_rerank`
- `coarse_bits=1`
- `rerank=heap_f32`
- `nlists=1024`
- Initial index reloption `nprobe=32`
- `training_sample_rows=50000`

Cells:

- `source/f32`: widths `32,64,128,256`
- `index/f16`: widths `32,64,128,256`
- `index/rabitq4`: widths `32,64,128,256`
- `index/rabitq8`: widths `32,64,128,256`
- `index/turboquant`: widths `32,64,128,256`

Recall and latency sweeps used `nprobe=8,16,32,64,128,200`.

## Commands

Audit:

```sh
target/release/ecaz bench suite audit --config reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/task111h-1m-rerank-format-width-v7-shared-suite.json --log-file reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/suite-audit.log
```

Run:

```sh
target/release/ecaz bench suite run --config reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/task111h-1m-rerank-format-width-v7-shared-suite.json --database task111h_rerank_1m_v7 --host /home/peter/.pgrx --port 28818 --artifact-dir reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts --log-file reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/suite-run.log
```

Status:

```sh
target/release/ecaz bench suite status --manifest reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/suite-manifest.json --database task111h_rerank_1m_v7 --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/suite-status.log
```

Report:

```sh
target/release/ecaz bench suite report --manifest reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/suite-manifest.json --results-output reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/results-report.jsonl --database task111h_rerank_1m_v7 --host /home/peter/.pgrx --port 28818 --log-file reviews/task-111h/028-rerank-suite-1m-v7-shared/artifacts/suite-report.log
```

## Durable Artifacts

| Artifact | sha256 | Notes |
| --- | --- | --- |
| `artifacts/task111h-1m-rerank-format-width-v7-shared-suite.json` | `2d12a65841197c84c846af77389cecccb9da38f032d84087afe1e6d04a729d2f` | SuiteConfig source of truth. |
| `artifacts/suite-manifest.json` | `a843c18313f7edfdef84178082c4cfdc1c175e2e7b761c63789224d6677dd57c` | Suite execution manifest. |
| `artifacts/results.jsonl` | `9ff8cbf082f14197f32fe25ce4ed6330d0c563cafb12f7a91d203fd79926705e` | Structured result rows from the run. |
| `artifacts/results-report.jsonl` | `9ff8cbf082f14197f32fe25ce4ed6330d0c563cafb12f7a91d203fd79926705e` | Structured result rows emitted by report. |
| `artifacts/suite-audit.log` | recorded in git blob | Audit log; passed 124 steps. |
| `artifacts/suite-dry-run.log` | recorded in git blob | Dry-run log before execution. |
| `artifacts/suite-run.log` | recorded in git blob | Full suite run log. |
| `artifacts/suite-status.log` | recorded in git blob | Status log; completed 124, failed 0, skipped 0. |
| `artifacts/suite-report.log` | recorded in git blob | Generated report log. |
| `artifacts/suite/*.log` | recorded in git blobs | Packet-local per-step load, recall, latency, storage, and drop logs referenced by suite manifest. |
| `artifacts/summary.md` | recorded in git blob | Human-readable summary derived from `results.jsonl`. |

## Key Result Lines

At `nprobe=32`:

- `source/f32 w64`: recall `0.9570`, formal latency mean `12.4 ms`, IVF index `226.8 MiB`.
- `index/f16 w64`: recall `0.9570`, formal latency mean `13.5 ms`, IVF index `3.2 GiB`.
- `index/rabitq4 w128`: recall `0.9100`, formal latency mean `12.5 ms`, IVF index `1014.4 MiB`.
- `index/rabitq8 w128`: recall `0.9210`, formal latency mean `14.1 ms`, IVF index `1.7 GiB`.
- `index/turboquant w128`: recall `0.9230`, formal latency mean `12.2 ms`, IVF index `1013.9 MiB`.

At `nprobe=200`:

- `source/f32 w64`: recall `0.9880`, formal latency mean `41.1 ms`.
- `source/f32 w128`: recall `0.9910`, formal latency mean `43.9 ms`.
- `index/f16 w128`: recall `0.9910`, formal latency mean `63.9 ms`.
- `index/rabitq8 w128`: recall `0.9520`, formal latency mean `47.9 ms`.
- `index/turboquant w128`: recall `0.9510`, formal latency mean `42.2 ms`.

The evidence supports the narrow conclusion that, in this suite, none of the quantized index-side rerank formats beat source/f32 on the combined recall/latency/storage tradeoff.
