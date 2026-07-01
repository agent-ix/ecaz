# Task 121 Packet 019 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/019-phase3-local-rabitq-sampled-pruning/`
- Head SHA: `f0fb7df1950dc2689503619780fca526587e1478`
- Suite: `task121-phase3-local-rabitq-sampled-pruning`
- Packet manifest written: `2026-06-25T18:25:00Z`
- Lane: local PG18, single PostgreSQL instance
- Database: `tqvector_bench_task121`
- Host/port: `/tmp:28818`
- PostgreSQL: `PostgreSQL 18.3 on x86_64-pc-linux-gnu`
- Build profile: `release`
- Fixture: `data/staged-current/ec_real_10k_corpus.tsv`
- Fixture SHA from loader log: `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75`
- Query SHA from loader log: `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8`
- Storage format: `rabitq`
- Rerank mode: default
- Isolation: one table/index per scale/cell; no shared-table surface
- Status: 10 completed, 0 failed, 18 pending/stale after intentional interrupt

## Commands

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-audit.log
```

Dry run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json --database tqvector_bench_task121 --host /tmp --port 28818 --dry-run --manifest-output reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-dryrun-manifest.json --log-file reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-dryrun.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-manifest.json --results-output reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-results.jsonl --log-file reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-run.log
```

Status after interrupt:

```text
target/debug/ecaz bench suite status --manifest reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-manifest.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning-status.log
```

## Status

```text
[suite:task121-phase3-local-rabitq-sampled-pruning] completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=18
```

The `load-50k_b4_tr50_f8_block64.log` file was generated before the interrupt
and contains only pre-load inspection lines. It is not a result artifact and is
not cited by this packet.

The generated `truth-cache-10k-q200-k10.json` file is a local-only cache and is
not committed. The committed evidence is the truth-cache log and the compact
recall, latency, storage, and pipeline logs.

## Artifacts

- `summary-10k-sampled-pruning.md`: compact 10k result summary.
- `suite-phase3-local-rabitq-sampled-pruning.json`: SuiteConfig used for the run.
- `suite-phase3-local-rabitq-sampled-pruning-audit.log`: audit command output.
- `suite-phase3-local-rabitq-sampled-pruning-dryrun.log`: dry-run command output.
- `suite-phase3-local-rabitq-sampled-pruning-dryrun-manifest.json`: dry-run manifest.
- `suite-phase3-local-rabitq-sampled-pruning-run.log`: interrupted suite execution log.
- `suite-phase3-local-rabitq-sampled-pruning-manifest.json`: run manifest.
- `suite-phase3-local-rabitq-sampled-pruning-status.log`: status snapshot after interrupt.
- `pg18-restart-before-phase3.log`: PG18 restart log.
- `pg18-ready-before-phase3.log`: PG18 readiness probe log.
- `precheck-host.log`: host/version/GUC precheck.
- `load-10k_b4_tr50_f8_block64.log`: 10k load/build log.
- `storage-10k_b4_tr50_f8_block64.log`: 10k storage log.
- `truth-cache-10k-q200-k10.log`: truth-cache build log.
- `recall-10k_b4_tr50_f8_block64_off.log`: pruning-off recall sweep.
- `recall-10k_b4_tr50_f8_block64_sampled.log`: sampled-pruning recall sweep.
- `latency-10k_b4_tr50_f8_block64_off.log`: pruning-off cache-warm latency.
- `latency-10k_b4_tr50_f8_block64_sampled.log`: sampled-pruning cache-warm latency.
- `pipeline-10k_b4_tr50_f8_block64_off.log`: pruning-off pipeline counters.
- `pipeline-10k_b4_tr50_f8_block64_sampled.log`: sampled-pruning pipeline counters.

## Key Result Lines

Storage:

```text
10k b4/tr50/f8 block64: index=42.1 MiB index_per_row=4415.5 B total=201.2 MiB total_per_row=21094.4 B
```

Recall:

```text
off:     r@8=0.9945 r@16=0.9980 r@32=0.9985 r@48=0.9995 r@64=1.0000 r@96=1.0000
sampled: r@8=0.9945 r@16=0.9980 r@32=0.9985 r@48=0.9995 r@64=1.0000 r@96=1.0000
```

Latency p50:

```text
off:     p50@8=68.4 ms p50@16=102.7 ms p50@32=144.0 ms p50@48=202.9 ms p50@64=254.0 ms p50@96=340.5 ms
sampled: p50@8=72.8 ms p50@16=102.5 ms p50@32=146.6 ms p50@48=210.7 ms p50@64=248.6 ms p50@96=285.6 ms
```

Pipeline counters:

```text
off:     candidates@32=2640327 candidates@48=3832280 candidates@64=5024981 candidates@96=7463419 p50@96=342.133 ms recall@96=1.0000
sampled: candidates@32=2640327 candidates@48=3832280 candidates@64=4923248 candidates@96=5121349 p50@96=283.740 ms recall@96=1.0000
```
