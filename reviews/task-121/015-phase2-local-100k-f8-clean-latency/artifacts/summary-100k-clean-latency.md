# 100k f8 clean latency summary

Suite: `task121-phase2-local-100k-f8-clean-latency`

Scope: local single-PG clean latency for two 100k f8 cells. This is not AWS and
not the local multi-node Phase 0 lane.

| nprobe | b0_tr10_f8 p50 | b0_tr10_f8 p95 | b1_tr50_f8 p50 | b1_tr50_f8 p95 | p50 ratio | p95 ratio |
|---:|---:|---:|---:|---:|---:|---:|
| 24 | 821.6 ms | 930.3 ms | 1420.0 ms | 1713.1 ms | 1.73x | 1.84x |
| 32 | 1102.9 ms | 1204.8 ms | 1776.2 ms | 1956.4 ms | 1.61x | 1.62x |
| 48 | 1767.4 ms | 1935.1 ms | 2508.0 ms | 2774.4 ms | 1.42x | 1.43x |
| 64 | 2326.1 ms | 2526.3 ms | 3129.7 ms | 3502.1 ms | 1.35x | 1.39x |
| 96 | 3538.6 ms | 3718.6 ms | 4193.1 ms | 4455.4 ms | 1.18x | 1.20x |

Interim read: the b1/tr50 f8 candidate is recall-promising in the earlier
pipeline evidence, but the clean latency cost is substantial at fixed nprobe,
especially at the lower nprobe range where recall gains were most valuable.

Still owed: 50k b2/b4, broader 100k recall, clean latency for any additional
finalist cells, Phase 3 scan-efficiency A/B, and Phase 4 Pareto verdict.
