# Task 121 review request: Phase 2 local 100k b8 and clean latency follow-up

## Scope

This packet responds to packet 017 reviewer feedback by extending the 100k
boundary knee from b4 to b8 and by adding clean cache-warm latency rows for
the b4 and b8 finalist cells.

Covered cells:

- new b8 rows: `b8/tr50/f8`, `b8/tr50/f16`
- clean latency rows: `b4/tr50/f8`, `b4/tr50/f16`, `b8/tr50/f8`, `b8/tr50/f16`
- fixed: `nlists=128`, top graph enabled, top graph degree 32, top graph
  search list size 96, `storage_format=rabitq`

This is local-only evidence on one PG18 PostgreSQL instance. It is not
multi-node evidence, not AWS evidence, and not Task 121 closeout evidence.

## Validation

Audit:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /tmp --port 28818 bench suite audit --config reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup.json --log-file reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup-audit.log
```

Run:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /tmp --port 28818 bench suite run --config reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup.json --manifest-output reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup-manifest.json --results-output reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup-results.jsonl --log-file reviews/task-121/018-phase2-local-100k-b8-latency-followup/artifacts/suite-phase2-local-100k-b8-latency-followup.log
```

The final suite manifest reports 12 steps, all succeeded:

```text
succeeded=12
```

Both b8 pipeline cells wrote the expected diagnostic row shape:
1800 funnel rows and 10800 stage-containment rows. The streamed JSONL files
are large local diagnostics and are intentionally not committed; the committed
evidence is the compact per-cell logs plus the structured suite results JSONL.

## Result

b8 closes the remaining 100k recall gap, but the cost is high. Both b8 cells
reach recall 1.0000 by nprobe 64 and stay at 1.0000 at nprobe 96. Compared
with packet 017 b4/tr50, the extra b8 recall is modest: b8/f16 improves the
selected checkpoints to r@8=0.9680, r@16=0.9900, r@32=0.9980, r@48=0.9990,
r@64=1.0000, and r@96=1.0000.

The clean cache-warm latency rows make the tradeoff clear. b8 roughly doubles
low-nprobe p50 latency versus b4 and remains slower at high nprobe. b4/f8 and
b4/f16 remain close to each other; b8/f16 is not a clean latency win over
b8/f8.

Storage also argues against b8 as the default finalist: b8 indexes are about
704.7-704.8 MiB, or 7389-7390 B/index-row, versus packet 017 b4 indexes around
392.2-392.3 MiB.

Selected b8 pipeline table:

| cell | r@4 | r@8 | r@16 | r@32 | r@48 | r@64 | r@96 | p50@32 | p50@96 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| b8_tr50_f8 | 0.8980 | 0.9630 | 0.9830 | 0.9970 | 0.9985 | 1.0000 | 1.0000 | 3477.841 ms | 5032.966 ms |
| b8_tr50_f16 | 0.8995 | 0.9680 | 0.9900 | 0.9980 | 0.9990 | 1.0000 | 1.0000 | 3528.944 ms | 5058.282 ms |

Clean latency table:

| cell | p50@8 | p50@16 | p50@32 | p50@48 | p50@96 |
|---|---:|---:|---:|---:|---:|
| b4_tr50_f8 | 955.0 ms | 1629.8 ms | 2699.4 ms | 3451.9 ms | 4730.6 ms |
| b4_tr50_f16 | 984.3 ms | 1660.4 ms | 2732.8 ms | 3497.3 ms | 4708.2 ms |
| b8_tr50_f8 | 1681.7 ms | 2661.6 ms | 3595.2 ms | 4250.1 ms | 5215.1 ms |
| b8_tr50_f16 | 1673.1 ms | 2565.5 ms | 3624.8 ms | 4294.1 ms | 5283.8 ms |

## Recommendation

Do not extend Phase 2 local 100k boundary replication to b16 before Phase 3
unless a reviewer specifically wants a wall-only datapoint. b8 proves that the
recall knee can be saturated, but it also shows the storage and latency slope
is too steep for the default candidate. The practical Phase 3 candidate remains
the b4/tr50 family, with fanout 8 versus 16 still close enough to carry both
or choose by the Phase 3 A/B budget.

## Remaining Work

This packet does not close Task 121. Still owed:

- Phase 3 scan-efficiency A/B at 10k/50k/100k for the recall-recovered candidate
- Phase 4 Pareto/verdict

## Artifacts

- `artifacts/manifest.md`
- `artifacts/summary-100k-b8-latency-followup.md`
- `artifacts/suite-phase2-local-100k-b8-latency-followup.json`
- `artifacts/suite-phase2-local-100k-b8-latency-followup-audit.log`
- `artifacts/suite-phase2-local-100k-b8-latency-followup.log`
- `artifacts/suite-phase2-local-100k-b8-latency-followup-manifest.json`
- `artifacts/suite-phase2-local-100k-b8-latency-followup-results.jsonl`
- `artifacts/precheck-host.log`
- `artifacts/load-100k_b8_tr50_f*.log`
- `artifacts/storage-100k_b8_tr50_f*.log`
- `artifacts/pipeline-100k_b8_tr50_f*.log`
- `artifacts/latency-100k_b*_tr50_f*.log`
- `artifacts/truth-cache-100k-q200-k10.log`
