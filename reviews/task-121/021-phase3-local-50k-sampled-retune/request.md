# Task 121 review request: Phase 3 local 50k sampled pruning retune

## Scope

Packet 020 showed the initial 50k sampled global pruning policy was too
aggressive: it improved high-nprobe runtime but dropped saturated-checkpoint
recall from 1.0000 to 0.9995. This packet tests one conservative retune on the
same packet 020 50k b4/tr50/f8 RaBitQ block-summary index:

```text
max_global_blocks=2048
global_probe_blocks=4096
sample_rows_per_block=4
sample_summary_prior_weight=0.8
summary_radius_weight=0.25
route_prior_weight=0.0
```

This is a recall-only retune packet. It does not close Phase 3.

## Validation

Audit:

```text
target/debug/ecaz bench suite audit --config reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune.json --database tqvector_bench_task121 --host /tmp --port 28818 --log-file reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-audit.log
```

Run:

```text
target/debug/ecaz bench suite run --config reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune.json --database tqvector_bench_task121 --host /tmp --port 28818 --manifest-output reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-manifest.json --results-output reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-results.jsonl --log-file reviews/task-121/021-phase3-local-50k-sampled-retune/artifacts/suite-phase3-local-50k-sampled-retune-run.log
```

Status:

```text
completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

The retuned policy restores 50k recall 1.0000 at all saturated checkpoints:

| nprobe | packet 020 off recall@10 | packet 020 g1024/p2048/r4 recall@10 | packet 021 g2048/p4096/r4 recall@10 |
|---:|---:|---:|---:|
| 48 | 1.0000 | 0.9995 | 1.0000 |
| 64 | 1.0000 | 0.9995 | 1.0000 |
| 96 | 1.0000 | 0.9995 | 1.0000 |

Mean query time comparison:

| nprobe | packet 020 off | packet 020 g1024/p2048/r4 | packet 021 g2048/p4096/r4 |
|---:|---:|---:|---:|
| 48 | 1423.23 ms | 1115.16 ms | 1421.46 ms |
| 64 | 1625.24 ms | 1200.44 ms | 1605.34 ms |
| 96 | 2045.17 ms | 1261.73 ms | 1786.64 ms |

The retune keeps a modest nprobe 96 runtime win versus pruning off while
recovering the one lost recall trial seen in packet 020.

## Recommendation

Carry `g2048/p4096/r4` forward as the 50k sampled-pruning candidate for
latency and pipeline A/B. Do not carry the packet 020 `g1024/p2048/r4` policy
forward except as the aggressive/lossy bound.

Still owed for Task 121 Phase 3:

- 50k latency and pipeline A/B for `g2048/p4096/r4`
- 100k recall retune plus latency/storage/pipeline
- 10k/50k/100k final scan-efficiency A/B summary
- default/TurboQuant block-summary coverage or explicit implementation-gap
  decision

## Artifacts

- `artifacts/manifest.md`
- `artifacts/summary-50k-sampled-retune.md`
- `artifacts/suite-phase3-local-50k-sampled-retune.json`
- `artifacts/suite-phase3-local-50k-sampled-retune-audit.log`
- `artifacts/suite-phase3-local-50k-sampled-retune-dryrun.log`
- `artifacts/suite-phase3-local-50k-sampled-retune-dryrun-manifest.json`
- `artifacts/suite-phase3-local-50k-sampled-retune-run.log`
- `artifacts/suite-phase3-local-50k-sampled-retune-manifest.json`
- `artifacts/suite-phase3-local-50k-sampled-retune-results.jsonl`
- `artifacts/suite-phase3-local-50k-sampled-retune-status.log`
- `artifacts/precheck-host.log`
- `artifacts/truth-cache-50k-q200-k10.log`
- `artifacts/recall-50k_b4_tr50_f8_block64_sampled_loose_g2048_p4096_r4.log`
