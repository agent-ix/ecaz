# Task 122 / 005 SPIRE prune release suite manifest

- Head SHA: `3bdc137ff` (`Add Task 122 SPIRE prune A/B suite packet`)
- Code SHA under measurement: `aa799704b` (`Gate SPIRE pre-materialization prune`)
- Task bucket: `reviews/task-122/005-spire-prune-release-suite/`
- Timestamp: `2026-06-27T14:07:08Z`
- Lane: local PG18 release, staged `ec_real_10k`, `ec_real_50k`, `ec_real_100k`
- Fixture: `data/staged-current/ec_real_{10k,50k,100k}_corpus.tsv`,
  `data/staged-current/ec_real_{10k,50k,100k}_queries.tsv`,
  `data/staged-current/ec_real_{10k,50k,100k}_manifest.json`
- Access method: `ec_spire`
- Quant/storage formats: `turboquant`, `rabitq`
- Rerank mode: `rerank_width=25`, `ec_spire.max_candidate_rows=25`
- Isolation: one index/table prefix per quant and scale

## Setup

Release install:

```sh
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config \
  > reviews/task-122/005-spire-prune-release-suite/artifacts/cargo-pgrx-install-release.log 2>&1
```

PG18 was restarted after the release install before the backend / GUC check.

Backend / GUC check:

```sh
/Users/peter/.cargo/bin/ecaz dev sql \
  --pg 18 \
  --db tqvector_bench \
  --socket-dir /Users/peter/.pgrx \
  --raw \
  --sql "SELECT ecaz_build_profile(); SELECT current_setting('ec_spire.pre_materialization_prune') AS pre_materialization_prune;" \
  --log-output reviews/task-122/005-spire-prune-release-suite/artifacts/guc-check-release.log
```

`guc-check-release.log` shows `ecaz_build_profile()` = `release` and
`ec_spire.pre_materialization_prune` = `on`.

## Suite

Config: `task122-spire-prune-release-suite.json`

Audit:

```sh
/Users/peter/.cargo/bin/ecaz bench suite audit \
  --config reviews/task-122/005-spire-prune-release-suite/artifacts/task122-spire-prune-release-suite.json \
  --log-file reviews/task-122/005-spire-prune-release-suite/artifacts/suite-audit-release.log
```

Dry run:

```sh
/Users/peter/.cargo/bin/ecaz bench suite run \
  --config reviews/task-122/005-spire-prune-release-suite/artifacts/task122-spire-prune-release-suite.json \
  --dry-run \
  --log-file reviews/task-122/005-spire-prune-release-suite/artifacts/suite-dry-run-release.log
```

Release suite:

```sh
/Users/peter/.cargo/bin/ecaz bench suite run \
  --config reviews/task-122/005-spire-prune-release-suite/artifacts/task122-spire-prune-release-suite.json \
  --host /Users/peter/.pgrx \
  --port 28818 \
  --log-file reviews/task-122/005-spire-prune-release-suite/artifacts/suite-run-release.log
```

Primary structured artifacts:

- `suite/suite-manifest.json`
- `suite/results.jsonl`

`suite/suite-manifest.json` records backend `build_profile=release` and all 36
steps as `succeeded`.

## Recall and latency

All rows use `nprobe=24`, `rerank_width=25`, and 100 queries.

| Scale | Lane | recall@k | NDCG | p50 | p95 | p99 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | TQ prune on | 1.0000 | 1.0000 | 2.15 ms | 2.44 ms | 2.84 ms |
| 10k | TQ prune off | 1.0000 | 1.0000 | 2.21 ms | 2.46 ms | 2.93 ms |
| 10k | RaBitQ | 1.0000 | 1.0000 | 2.14 ms | 2.35 ms | 2.84 ms |
| 50k | TQ prune on | 0.9450 | 0.9969 | 4.40 ms | 4.74 ms | 5.84 ms |
| 50k | TQ prune off | 0.9450 | 0.9969 | 4.51 ms | 4.98 ms | 6.01 ms |
| 50k | RaBitQ | 0.9450 | 0.9969 | 4.33 ms | 4.80 ms | 5.80 ms |
| 100k | TQ prune on | 0.8940 | 0.9893 | 6.30 ms | 6.92 ms | 8.02 ms |
| 100k | TQ prune off | 0.8940 | 0.9893 | 6.45 ms | 6.96 ms | 8.30 ms |
| 100k | RaBitQ | 0.8940 | 0.9893 | 6.39 ms | 6.93 ms | 8.05 ms |

## Pipeline counters

Candidate rows are from the SPIRE `candidates` pipeline stage.

| Scale | Lane | item_sum | ready_sum | blocked_sum | candidate_sum | heap_rerank_sum |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | TQ prune on | 8,495 | 2,500 | 249,055 | 8,495 | 0 |
| 10k | TQ prune off | 251,555 | 2,500 | 249,055 | 251,555 | 0 |
| 50k | TQ prune on | 11,796 | 2,500 | 522,567 | 11,796 | 0 |
| 50k | TQ prune off | 525,067 | 2,500 | 522,567 | 525,067 | 0 |
| 100k | TQ prune on | 10,517 | 2,500 | 763,994 | 10,517 | 0 |
| 100k | TQ prune off | 766,494 | 2,500 | 763,994 | 766,494 | 0 |

Heap rerank stage emitted `2,500` rows for both prune-on and prune-off at all
three scales.

## Storage

| Scale | Lane | SPIRE index size | Per row |
| --- | --- | ---: | ---: |
| 10k | TQ | 8.9 MiB | 931.4 B |
| 10k | RaBitQ | 9.0 MiB | 939.6 B |
| 50k | TQ | 41.4 MiB | 868.5 B |
| 50k | RaBitQ | 41.6 MiB | 872.9 B |
| 100k | TQ | 81.4 MiB | 854.0 B |
| 100k | RaBitQ | 81.7 MiB | 856.8 B |

## Interpretation

- The pre-materialization prune is recall-neutral in this suite: prune-on and
  prune-off recall/NDCG are identical at each scale.
- It substantially reduces materialized TQ candidate rows before truncation:
  29.6x at 10k, 44.5x at 50k, and 72.9x at 100k.
- Latency improves modestly versus prune-off, but does not establish a product
  win over RaBitQ at this fixed `nprobe=24`, `rerank_width=25` shape.
- The fixed 50k and 100k setting is not a high-recall closeout point:
  recall@k is `0.9450` at 50k and `0.8940` at 100k.
- Task 122 remains open. Next evidence needs a matched-recall matrix that
  sweeps candidate budget / nprobe / f32 rerank width for RaBitQ -> f32,
  TQ -> f32, and, where supported, RaBitQ -> TQ -> f32.
