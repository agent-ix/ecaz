# Task 121 review request: Phase 2 local 100k axis-fix matrix

## Scope

This packet completes the missing local 100k Phase 2 boundary/training/fanout
axis-fix matrix. It covers all 16 isolated 100k SPIRE cells:

- boundary replica count: 0, 1, 2, 4
- training sample rows: 10000, 50000
- recursive fanout: 8, 16
- fixed: `nlists=128`, top graph enabled, top graph degree 32, top graph
  search list size 96, `storage_format=rabitq`

This is local-only evidence on one PG18 PostgreSQL instance. It is not
multi-node evidence, not AWS evidence, and not Task 121 closeout evidence.

## Validation

Audit:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /tmp --port 28818 bench suite audit --config reviews/task-121/017-phase2-local-100k-axis-fix-run/artifacts/suite-phase2-local-100k-axis-fix-run.json
```

Run:

```text
target/debug/ecaz --database tqvector_bench_task121 --host /tmp --port 28818 bench suite run --config reviews/task-121/017-phase2-local-100k-axis-fix-run/artifacts/suite-phase2-local-100k-axis-fix-run.json --manifest-output reviews/task-121/017-phase2-local-100k-axis-fix-run/artifacts/suite-phase2-local-100k-axis-fix-run-manifest.json --results-output reviews/task-121/017-phase2-local-100k-axis-fix-run/artifacts/suite-phase2-local-100k-axis-fix-run-results.jsonl --log-file reviews/task-121/017-phase2-local-100k-axis-fix-run/artifacts/suite-phase2-local-100k-axis-fix-run-fresh.log
```

The final suite manifest reports 50 steps, all succeeded:

```text
succeeded=50
```

Each completed pipeline cell wrote the expected diagnostic row shape:
1800 funnel rows and 10800 stage-containment rows. The streamed JSONL files are
large local diagnostics and are intentionally not committed; the committed
evidence is the compact per-cell logs plus the structured suite results JSONL.

## Result

Boundary replication is the dominant recall lever at 100k, but it is expensive.
At nprobe 8, recall rises from the b0/tr10 baseline 0.7250 to 0.9135-0.9340
for b4 cells. At nprobe 32, b4 cells are at 0.9895-0.9920 recall, while b0
cells are 0.9310-0.9560 and b1/b2 cells sit between those. All b2 and b4 cells
reach recall 1.0000 by nprobe 96.

Training sample 50000 helps b0/b1/b2 low- and mid-nprobe recall, but it does
not change storage materially. Fanout 16 is not a clean win over fanout 8:
it sometimes improves recall slightly, but the effect is smaller than boundary
replication and not consistently cheaper.

The strongest 100k recall/cost shape in this packet is b4/tr50 with either
fanout. b4/tr50_f16 has the best selected recall checkpoints among b4 cells
(0.9340 at nprobe 8, 0.9755 at 16, 0.9970 at 48, 1.0000 at 96), but its p50
latency is effectively the same as b4/tr50_f8 at the high end. b4/tr50_f8 is
nearly identical at nprobe 4/8/96 and slightly lower at 16/48.

Storage grows almost linearly with boundary replica count:

| cell family | index size | index bytes/row | total table |
|---|---:|---:|---:|
| b0 | ~79.6-79.8 MiB | ~835-837 B | 1.6 GiB |
| b1 | ~157.8-157.9 MiB | ~1655-1656 B | 1.7 GiB |
| b2 | ~235.9-236.0 MiB | ~2474-2475 B | 1.8 GiB |
| b4 | ~392.2-392.3 MiB | ~4112-4113 B | 1.9 GiB |

Compact selected recall/latency table:

| cell | r@4 | r@8 | r@16 | r@32 | r@48 | r@96 | p50@32 | p50@96 | p95@96 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| b0_tr10_f8 | 0.5500 | 0.7250 | 0.8525 | 0.9310 | 0.9645 | 0.9975 | 1080.340 ms | 3332.217 ms | 3851.251 ms |
| b0_tr10_f16 | 0.5760 | 0.7480 | 0.8605 | 0.9400 | 0.9680 | 0.9960 | 1062.517 ms | 3269.148 ms | 3696.316 ms |
| b0_tr50_f8 | 0.6240 | 0.7785 | 0.8810 | 0.9455 | 0.9725 | 0.9960 | 1069.361 ms | 3299.373 ms | 3674.026 ms |
| b0_tr50_f16 | 0.6170 | 0.7795 | 0.8945 | 0.9560 | 0.9740 | 0.9965 | 1035.273 ms | 3239.536 ms | 3596.714 ms |
| b1_tr10_f8 | 0.6870 | 0.8365 | 0.9235 | 0.9735 | 0.9870 | 0.9995 | 1720.054 ms | 3967.281 ms | 4607.560 ms |
| b1_tr10_f16 | 0.7025 | 0.8575 | 0.9330 | 0.9750 | 0.9900 | 0.9990 | 1604.568 ms | 3787.897 ms | 4209.942 ms |
| b1_tr50_f8 | 0.7480 | 0.8625 | 0.9355 | 0.9760 | 0.9880 | 0.9990 | 1560.862 ms | 3815.621 ms | 4376.480 ms |
| b1_tr50_f16 | 0.7300 | 0.8645 | 0.9475 | 0.9810 | 0.9885 | 0.9995 | 1521.840 ms | 3710.127 ms | 4156.546 ms |
| b2_tr10_f8 | 0.7490 | 0.8730 | 0.9465 | 0.9835 | 0.9925 | 1.0000 | 1997.621 ms | 4125.518 ms | 4564.965 ms |
| b2_tr10_f16 | 0.7495 | 0.8875 | 0.9555 | 0.9865 | 0.9950 | 1.0000 | 2025.861 ms | 4206.851 ms | 4681.396 ms |
| b2_tr50_f8 | 0.7880 | 0.8945 | 0.9535 | 0.9825 | 0.9920 | 1.0000 | 1935.449 ms | 4155.910 ms | 4534.717 ms |
| b2_tr50_f16 | 0.7760 | 0.9035 | 0.9630 | 0.9850 | 0.9930 | 1.0000 | 1942.887 ms | 4197.764 ms | 4690.854 ms |
| b4_tr10_f8 | 0.8230 | 0.9135 | 0.9655 | 0.9915 | 0.9955 | 1.0000 | 2706.355 ms | 4511.855 ms | 5065.399 ms |
| b4_tr10_f16 | 0.8210 | 0.9280 | 0.9750 | 0.9920 | 0.9970 | 1.0000 | 2629.091 ms | 4514.066 ms | 4768.395 ms |
| b4_tr50_f8 | 0.8405 | 0.9330 | 0.9670 | 0.9895 | 0.9945 | 1.0000 | 2544.873 ms | 4498.213 ms | 5006.301 ms |
| b4_tr50_f16 | 0.8385 | 0.9340 | 0.9755 | 0.9915 | 0.9970 | 1.0000 | 2589.751 ms | 4504.694 ms | 5164.273 ms |

## Remaining Work

This packet does not close Task 121. Still owed:

- credible clean latency re-measurement on a quiesced host for any finalist
  100k cells
- Phase 3 scan-efficiency A/B at 10k/50k/100k for the recall-recovered
  candidate
- Phase 4 Pareto/verdict

## Artifacts

- `artifacts/manifest.md`
- `artifacts/summary-100k-axis-fix-run.md`
- `artifacts/suite-phase2-local-100k-axis-fix-run.json`
- `artifacts/suite-phase2-local-100k-axis-fix-run-audit.log`
- `artifacts/suite-phase2-local-100k-axis-fix-run-fresh.log`
- `artifacts/suite-phase2-local-100k-axis-fix-run-manifest.json`
- `artifacts/suite-phase2-local-100k-axis-fix-run-results.jsonl`
- `artifacts/precheck-host.log`
- `artifacts/load-100k_*.log`
- `artifacts/storage-100k_*.log`
- `artifacts/pipeline-100k_*.log`
- `artifacts/truth-cache-100k-q200-k10.log`
