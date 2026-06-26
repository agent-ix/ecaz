---
head_sha: 90a15d1c8be12194ab5a3c2e0d7064a7f8f7a18a
task_bucket: reviews/task-89
packet: reviews/task-89/005-ivf-tqplus-insert-drift
timestamp_utc: 2026-06-26T03:20:00Z
---

# Artifact Manifest

## Scope

Task 89 IVF TQ+ streaming-insert drift evidence. This packet measures live
post-build insert recall against full-rebuild recall at 10%, 25%, and 50%
insert ratios.

## Suite Shape

### `suite.json`

- Lane: local PG18 `ec_ivf`
- Fixture: staged DBPedia 50k source reservoir
- Live table: 10k rows, TQ+ index built once, then post-build inserts
- Full-rebuild baselines:
  - 11k rows for 10% insert comparison
  - 12.5k rows for 25% insert comparison
  - 15k rows for 50% insert comparison
- Storage format: `storage_format=turboquant`
- TQ+ option: `turboquant_calibration=tqplus_experimental`
- Recall cell: `recall@10`, `nprobe=48`, 200 queries
- Isolation: one table/index prefix per live or rebuild surface
- Suite command:
  `./target/debug/ecaz bench suite run --config reviews/task-89/005-ivf-tqplus-insert-drift/suite.json --host /Users/peter/.pgrx --port 28818`

## Validation Artifacts

### `suite-audit.log`

- Command:
  `./target/debug/ecaz bench suite audit --config reviews/task-89/005-ivf-tqplus-insert-drift/suite.json`
- Result: `audit passed: 14 steps`

### `suite-dry-run.log`

- Command:
  `./target/debug/ecaz bench suite run --config reviews/task-89/005-ivf-tqplus-insert-drift/suite.json --dry-run --manifest-output reviews/task-89/005-ivf-tqplus-insert-drift/artifacts/suite-manifest-dry-run.json`
- Result: expanded all 14 steps and wrote
  `artifacts/suite-manifest-dry-run.json`.

### `suite-run.log`

- Command:
  `./target/debug/ecaz bench suite run --config reviews/task-89/005-ivf-tqplus-insert-drift/suite.json --host /Users/peter/.pgrx --port 28818`
- Result: completed all 14 steps and wrote:
  - `artifacts/suite/results.jsonl`
  - `artifacts/suite/suite-manifest.json`

## Key Results

### Source Reservoir

- Loaded staged DBPedia 50k into `task89_drift_source50k_tq`.
- Corpus rows: 50,000.
- Query rows: 1,000.
- Source load total: 94.35 s.

### Live Insert Row Counts

| step | inserted rows | live rows after insert | id range |
| --- | ---: | ---: | --- |
| `insert-live10pct` | 1,000 | 11,000 | `0..10999` |
| `insert-live25pct` | 1,500 | 12,500 | `0..12499` |
| `insert-live50pct` | 2,500 | 15,000 | `0..14999` |

### Drift Recall

| insert ratio | live rows | live recall@10 | rebuild recall@10 | live-minus-rebuild | threshold result |
| --- | ---: | ---: | ---: | ---: | --- |
| 10% | 11,000 | 0.9265 | 0.9310 | -0.0045 | informational |
| 25% | 12,500 | 0.9230 | 0.9235 | -0.0005 | pass, <= 0.005 |
| 50% | 15,000 | 0.9245 | 0.9220 | +0.0025 | pass, <= 0.010 |

The 25% and 50% cells satisfy Task 89's initial drift thresholds. The 10%
cell is below the stricter 25% threshold as well.

## Artifact Index

- `suite-audit.log`
- `suite-dry-run.log`
- `suite-manifest-dry-run.json`
- `suite-run.log`
- `artifacts/suite/results.jsonl`
- `artifacts/suite/suite-manifest.json`
- `artifacts/suite/load-source50k-tq.log`
- `artifacts/suite/create-live10-tqplus.log`
- `artifacts/suite/insert-live10pct.log`
- `artifacts/suite/create-rebuild11k-tqplus.log`
- `artifacts/suite/recall-live10pct.log`
- `artifacts/suite/recall-rebuild11k.log`
- `artifacts/suite/insert-live25pct.log`
- `artifacts/suite/create-rebuild12500-tqplus.log`
- `artifacts/suite/recall-live25pct.log`
- `artifacts/suite/recall-rebuild12500.log`
- `artifacts/suite/insert-live50pct.log`
- `artifacts/suite/create-rebuild15k-tqplus.log`
- `artifacts/suite/recall-live50pct.log`
- `artifacts/suite/recall-rebuild15k.log`
