# Review Request: Task 124 Phase 6 local cache-evict validation

## Scope

This packet is the Phase 6 IO-sensitive validation requested in `reviews/task-124/011-tq-selected-payload-slab/feedback/2026-06-29-01-reviewer.md`.

It stays on TurboQuant. It does not add another knob/format/allocation sweep. Packets 012-014 already tested the named structural slices and reverted them after negative A/B evidence:

- 012 top-k fusion: negative, reverted.
- 013 compact group header: negative, reverted.
- 014 direct-slot rerank: negative, reverted.

This packet adds one CLI support change, committed separately as `3be1ba32e`: local macOS relation cache eviction now uses `fcntl(F_NOCACHE)` so `ecaz dev evict-relation-cache` can run on this host instead of failing as Linux-only.

## Evidence

Primary artifacts:

- `artifacts/manifest.md`
- `artifacts/task124-tq-phase6-local-cache-evict-100k-suite.json`
- `artifacts/suite-manifest-r2.json`
- `artifacts/results-r2.jsonl`
- `artifacts/suite-status-r2.log`
- `artifacts/suite-report-r2.log`
- `artifacts/report-results-r2.jsonl`
- `artifacts/cache-evict-summary.md`
- `artifacts/local-cache-evict-100k/latency-100k-f32-w100-cache-evict.log`
- `artifacts/local-cache-evict-100k/latency-100k-tq-w75-g50-final15-cache-evict.log`
- `artifacts/local-cache-evict-100k/recall-100k-f32-w100.log`
- `artifacts/local-cache-evict-100k/recall-100k-tq-w75-g50-final15.log`
- `artifacts/local-cache-evict-100k/storage-100k-f32-w100.log`
- `artifacts/local-cache-evict-100k/storage-100k-tq-w75-g50-final15.log`

The suite completed all 10 steps:

```text
[suite:task124-tq-phase6-local-cache-evict-100k-suite] completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Result

Recall is equal to f32 at 100k:

| variant | nprobe | recall@k | ndcg@k |
| --- | ---: | ---: | ---: |
| f32 | 32 | 0.9730 | 0.9969 |
| TQ | 32 | 0.9730 | 0.9969 |
| f32 | 64 | 1.0000 | 1.0000 |
| TQ | 64 | 1.0000 | 1.0000 |

Latency is not a product win under local macOS relation `F_NOCACHE`:

| variant | nprobe | mean | p50 | p95 | p99 |
| --- | ---: | ---: | ---: | ---: | ---: |
| f32 | 32 | 6.39 ms | 5.74 ms | 9.53 ms | 13.8 ms |
| TQ | 32 | 7.37 ms | 6.76 ms | 10.9 ms | 14.3 ms |
| f32 | 64 | 9.20 ms | 9.01 ms | 11.5 ms | 11.8 ms |
| TQ | 64 | 9.44 ms | 9.24 ms | 9.98 ms | 12.8 ms |

Storage remains the wall:

| variant | total | indexes | ec_ivf index | ec_ivf index per row |
| --- | ---: | ---: | ---: | ---: |
| f32 | 1.6 GiB | 24.6 MiB | 22.5 MiB | 235.8 B |
| TQ | 1.7 GiB | 103.0 MiB | 100.8 MiB | 1057.2 B |

TurboQuant scoring is still full SIMD:

```text
tq nprobe=32 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
tq nprobe=64 turboquant isa=neon candidates=7500 scalar_candidates=0 width_ge32=100
```

## Interpretation

This resolves the reviewer-requested Phase 6 local IO-sensitive check as negative/mixed:

- TQ recall matches f32.
- TQ does not improve p50/mean at either nprobe.
- TQ improves p95 at nprobe64 only, while p50 and p99 remain worse.
- The persisted TQ index is still about 4.5x the f32 ec_ivf index size.
- The scorer path is SIMD; the blocker is not scalar TurboQuant scoring.

I recommend Shelve for Task 124 unless a reviewer specifically requires a remote true-cold run. Local cloud profiles were unavailable during this packet, and `/usr/sbin/purge` requires privileges on this host, so this run uses per-relation macOS `F_NOCACHE` rather than a global OS cache purge.
