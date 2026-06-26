---
head_sha: a9f3dd7f75da1b9b535af5bf7ac4c20d2b5a495c
task_bucket: reviews/task-89
packet: reviews/task-89/006-ivf-tqplus-cross-corpus
timestamp_utc: 2026-06-26T04:10:00Z
---

# Artifact Manifest

## Scope

Task 89 IVF TQ+ cross-corpus evidence. This packet measures IVF TQ+ against
baseline TurboQuant on a deterministic synthetic non-DBPedia unit-sphere
distribution.

## Suite Shape

### `suite.json`

- Lane: local PG18 `ec_ivf`
- Fixture: deterministic synthetic unit-sphere vectors
- Corpus rows: 10,000
- Query rows: 200
- Dimensions: 1536
- Storage format: `storage_format=turboquant`
- Variants:
  - baseline TurboQuant
  - TQ+ via `turboquant_calibration=tqplus_experimental`
- Matrix: generate, load, recall@10, latency, storage

## Validation Artifacts

- `suite-audit.log`: suite audit passed with 10 steps.
- `suite-manifest-dry-run.json`: dry-run expansion for the checked-in suite.
- `suite/suite-manifest.json`: live suite manifest.
- `suite/results.jsonl`: structured live result stream.
- `suite/*.log`: command logs for load, recall, latency, and storage.

The generated TSV inputs live under `artifacts/generated/` and are intentionally
not committed. They are deterministic suite outputs:

- Corpus: 10,000 rows, dim 1536, seed 8906
- Queries: 200 rows, dim 1536, seed 8907

## Commands

- `./target/debug/ecaz bench suite audit --config reviews/task-89/006-ivf-tqplus-cross-corpus/suite.json`
- `./target/debug/ecaz bench suite run --config reviews/task-89/006-ivf-tqplus-cross-corpus/suite.json --dry-run --manifest-output reviews/task-89/006-ivf-tqplus-cross-corpus/artifacts/suite-manifest-dry-run.json`
- `./target/debug/ecaz bench suite run --config reviews/task-89/006-ivf-tqplus-cross-corpus/suite.json --host /Users/peter/.pgrx --port 28818`

## Key Results

### Recall

| nprobe | TQ recall@10 | TQ+ recall@10 | Delta |
| ---: | ---: | ---: | ---: |
| 16 | 0.3800 | 0.3755 | -0.45 pp |
| 32 | 0.6075 | 0.5780 | -2.95 pp |
| 48 | 0.7675 | 0.7175 | -5.00 pp |
| 64 | 0.8610 | 0.7880 | -7.30 pp |

### Latency

| nprobe | TQ p50 | TQ+ p50 | TQ p95 | TQ+ p95 |
| ---: | ---: | ---: | ---: | ---: |
| 16 | 1.72 ms | 4.07 ms | 2.27 ms | 4.43 ms |
| 32 | 3.99 ms | 5.76 ms | 4.57 ms | 6.04 ms |
| 48 | 3.99 ms | 8.38 ms | 4.60 ms | 8.73 ms |
| 64 | 4.24 ms | 10.9 ms | 4.66 ms | 11.2 ms |

These latency numbers are recorded for traceability but are not gate-decision
evidence: current TQ+ scoring is scalar-only, while baseline TurboQuant can use
tiled/SIMD scoring. The public-shape decision should use recall, storage, and
drift unless a comparable TQ+ scorer lands.

### Storage

- Baseline IVF TQ total: 168.5 MiB, 17666.9 B/row total, 989.6 B/index row.
- IVF TQ+ total: 168.5 MiB, 17668.5 B/row total, 991.2 B/index row.

## Interpretation

The synthetic non-DBPedia gate shows recall loss at every measured `nprobe`.
This satisfies the Task 89 stop condition for a cross-corpus regression and
should feed the public-shape gate as evidence against promoting TQ+ as a
production option or separate format in its current form. The latency rows
remain diagnostic only until TQ+ has scorer parity with baseline TurboQuant.
