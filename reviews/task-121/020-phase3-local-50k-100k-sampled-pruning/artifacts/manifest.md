# Task 121 Packet 020 Artifact Manifest

## Packet

- Task: 121
- Packet: `reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/`
- Head SHA: `548936815e208cf007d0ba4e3985772fad21bcd3`
- Source SuiteConfig: `reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json`
- Suite: `task121-phase3-local-rabitq-sampled-pruning`
- Packet manifest written: `2026-06-25T19:10:00Z`
- Lane: local PG18, single PostgreSQL instance
- Database: `tqvector_bench_task121`
- Host/port: `/tmp:28818`
- PostgreSQL: `PostgreSQL 18.3 on x86_64-pc-linux-gnu`
- Build profile: `release`
- Fixture: `data/staged-current/ec_real_50k_corpus.tsv`
- Fixture SHA from loader log: `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133`
- Query SHA from loader log: `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3`
- Storage format: `rabitq`
- Rerank mode: default
- Isolation: one table/index per scale/cell; no shared-table surface
- Status: 5 completed, 0 failed, 10 skipped, 13 pending/stale after intentional interrupt

## Commands

Dry run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json --only-tag 50k --only-tag 100k --artifact-dir reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts --database tqvector_bench_task121 --host /tmp --port 28818 --dry-run --manifest-output reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-dryrun-manifest.json --log-file reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-dryrun.log
```

Precheck:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /tmp --port 28818 dev sql --pg 18 --db tqvector_bench_task121 --socket-dir /tmp --raw --sql "LOAD 'ecaz'; SELECT now() AS captured_at, version() AS postgres_version, ecaz_build_profile() AS ecaz_build_profile; SELECT name, setting FROM pg_settings WHERE name IN ('ec_spire.leaf_block_rows', 'ec_spire.leaf_block_summary_representatives', 'ec_spire.leaf_block_pruning_max_blocks_per_leaf', 'ec_spire.leaf_block_pruning_max_global_blocks', 'ec_spire.leaf_block_pruning_global_probe_blocks', 'ec_spire.leaf_block_pruning_sample_rows_per_block', 'ec_spire.leaf_block_pruning_sample_summary_prior_weight', 'ec_spire.leaf_block_pruning_summary_radius_weight', 'ec_spire.leaf_block_pruning_route_prior_weight') ORDER BY name;" --log-output reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/precheck-host.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/019-phase3-local-rabitq-sampled-pruning/artifacts/suite-phase3-local-rabitq-sampled-pruning.json --only-tag 50k --only-tag 100k --artifact-dir reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-manifest.json --results-output reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-results.jsonl --log-file reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-run.log
```

Status after interrupt:

```text
target/debug/ecaz bench suite status --manifest reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-manifest.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/020-phase3-local-50k-100k-sampled-pruning/artifacts/suite-phase3-local-50k-100k-sampled-pruning-status.log
```

## Status

```text
[suite:task121-phase3-local-rabitq-sampled-pruning] completed=5 failed=0 skipped=10 dry_run=0 missing_artifacts=0 stale=13
```

The generated `truth-cache-50k-q200-k10.json` file is a local-only cache and is
not committed. The committed evidence is the truth-cache log and the compact
storage/recall logs.

The suite was interrupted after `recall-50k_b4_tr50_f8_block64_sampled`
succeeded and before `latency-50k_b4_tr50_f8_block64_off` completed.

## Artifacts

- `summary-50k-recall-sampled-pruning.md`: compact 50k storage/recall summary.
- `suite-phase3-local-50k-100k-sampled-pruning-dryrun.log`: selected dry-run output.
- `suite-phase3-local-50k-100k-sampled-pruning-dryrun-manifest.json`: selected dry-run manifest.
- `suite-phase3-local-50k-100k-sampled-pruning-run.log`: interrupted selected suite execution log.
- `suite-phase3-local-50k-100k-sampled-pruning-manifest.json`: selected run manifest.
- `suite-phase3-local-50k-100k-sampled-pruning-status.log`: status snapshot after interrupt.
- `precheck-host.log`: host/version/GUC precheck.
- `load-50k_b4_tr50_f8_block64.log`: 50k load/build log.
- `storage-50k_b4_tr50_f8_block64.log`: 50k storage log.
- `truth-cache-50k-q200-k10.log`: truth-cache seed recall log.
- `recall-50k_b4_tr50_f8_block64_off.log`: pruning-off recall sweep.
- `recall-50k_b4_tr50_f8_block64_sampled.log`: sampled-pruning recall sweep.

## Key Result Lines

Storage:

```text
50k b4/tr50/f8 block64: index=203.4 MiB index_per_row=4265.2 B total=998.3 MiB total_per_row=20936.6 B
```

Recall:

```text
off:     r@8=0.9810 r@16=0.9905 r@32=0.9985 r@48=1.0000 r@64=1.0000 r@96=1.0000
sampled: r@8=0.9810 r@16=0.9905 r@32=0.9985 r@48=0.9995 r@64=0.9995 r@96=0.9995
```

Mean query time:

```text
off:     mean@8=371.82 ms mean@16=603.99 ms mean@32=1062.88 ms mean@48=1423.23 ms mean@64=1625.24 ms mean@96=2045.17 ms
sampled: mean@8=379.34 ms mean@16=614.80 ms mean@32=1040.89 ms mean@48=1115.16 ms mean@64=1200.44 ms mean@96=1261.73 ms
```
