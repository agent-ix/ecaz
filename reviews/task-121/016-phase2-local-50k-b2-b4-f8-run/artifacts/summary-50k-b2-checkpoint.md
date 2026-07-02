# Task 121 Phase 2 local 50k b2/b4 f8 checkpoint

This is local-only, single-PostgreSQL evidence. It is not AWS evidence and it is
not the Phase 0 local multi-node lane.

## Completed State

Shared suite status after the b2 breakpoint halt:

- completed: 12
- failed: 0
- pending: 2

Completed:

- all b2/b4 load steps
- all b2/b4 storage steps
- 50k truth-cache generation
- `pipeline-50k_b2_tr10_f8`
- `pipeline-50k_b2_tr50_f8`

Additional b4-only resume completed:

- `pipeline-50k_b4_tr10_f8`
- `pipeline-50k_b4_tr50_f8`

The b4/tr50-only resume wrote 1800 funnel rows and 10800 stage-containment
rows. The streamed JSONLs remain untracked; the compact log and suite JSONL are
packet evidence.

## Storage

| cell | boundary replicas | training rows | index | per-row index | table total |
|---|---:|---:|---:|---:|---:|
| b2_tr10_f8 | 2 | 10000 | 118.7 MiB | 2488.7 B | 913.6 MiB |
| b2_tr50_f8 | 2 | 50000 | 118.8 MiB | 2490.4 B | 913.7 MiB |
| b4_tr10_f8 | 4 | 10000 | 196.9 MiB | 4129.6 B | 991.9 MiB |
| b4_tr50_f8 | 4 | 50000 | 196.9 MiB | 4128.8 B | 991.8 MiB |

## Completed b2 Pipeline Results

| cell | nprobe | p50 | p95 | recall@10 |
|---|---:|---:|---:|---:|
| b2_tr10_f8 | 4 | 157.582 ms | 208.783 ms | 0.9205 |
| b2_tr10_f8 | 8 | 257.195 ms | 345.027 ms | 0.9525 |
| b2_tr10_f8 | 12 | 350.241 ms | 461.687 ms | 0.9690 |
| b2_tr10_f8 | 16 | 439.302 ms | 576.116 ms | 0.9730 |
| b2_tr10_f8 | 24 | 630.501 ms | 764.504 ms | 0.9865 |
| b2_tr10_f8 | 32 | 829.647 ms | 942.381 ms | 0.9950 |
| b2_tr10_f8 | 48 | 1125.506 ms | 1273.767 ms | 0.9970 |
| b2_tr10_f8 | 64 | 1361.775 ms | 1505.985 ms | 0.9995 |
| b2_tr10_f8 | 96 | 1743.187 ms | 1969.979 ms | 1.0000 |
| b2_tr50_f8 | 4 | 159.156 ms | 205.735 ms | 0.9385 |
| b2_tr50_f8 | 8 | 265.862 ms | 364.709 ms | 0.9680 |
| b2_tr50_f8 | 12 | 374.703 ms | 473.330 ms | 0.9765 |
| b2_tr50_f8 | 16 | 476.989 ms | 613.124 ms | 0.9810 |
| b2_tr50_f8 | 24 | 678.687 ms | 838.559 ms | 0.9950 |
| b2_tr50_f8 | 32 | 868.990 ms | 1040.976 ms | 0.9965 |
| b2_tr50_f8 | 48 | 1202.368 ms | 1354.413 ms | 0.9990 |
| b2_tr50_f8 | 64 | 1454.398 ms | 1603.588 ms | 0.9995 |
| b2_tr50_f8 | 96 | 1896.008 ms | 2091.639 ms | 1.0000 |

## Completed b4/tr10 Pipeline Results

| cell | nprobe | p50 | p95 | recall@10 |
|---|---:|---:|---:|---:|
| b4_tr10_f8 | 4 | 236.562 ms | 339.021 ms | 0.9575 |
| b4_tr10_f8 | 8 | 395.642 ms | 529.372 ms | 0.9725 |
| b4_tr10_f8 | 12 | 492.944 ms | 660.507 ms | 0.9840 |
| b4_tr10_f8 | 16 | 589.129 ms | 808.869 ms | 0.9890 |
| b4_tr10_f8 | 24 | 830.643 ms | 1032.950 ms | 0.9945 |
| b4_tr10_f8 | 32 | 1066.051 ms | 1210.029 ms | 0.9980 |
| b4_tr10_f8 | 48 | 1352.252 ms | 1523.077 ms | 0.9990 |
| b4_tr10_f8 | 64 | 1571.358 ms | 1793.543 ms | 1.0000 |
| b4_tr10_f8 | 96 | 1900.814 ms | 2199.261 ms | 1.0000 |

## Completed b4/tr50 Pipeline Results

| cell | nprobe | p50 | p95 | recall@10 |
|---|---:|---:|---:|---:|
| b4_tr50_f8 | 4 | 244.920 ms | 564.265 ms | 0.9650 |
| b4_tr50_f8 | 8 | 400.503 ms | 547.652 ms | 0.9810 |
| b4_tr50_f8 | 12 | 556.820 ms | 756.253 ms | 0.9865 |
| b4_tr50_f8 | 16 | 658.855 ms | 882.720 ms | 0.9905 |
| b4_tr50_f8 | 24 | 877.132 ms | 1107.089 ms | 0.9975 |
| b4_tr50_f8 | 32 | 1130.192 ms | 1410.404 ms | 0.9985 |
| b4_tr50_f8 | 48 | 1482.835 ms | 1740.106 ms | 1.0000 |
| b4_tr50_f8 | 64 | 1668.702 ms | 2202.986 ms | 1.0000 |
| b4_tr50_f8 | 96 | 2135.053 ms | 2647.988 ms | 1.0000 |

## Interim Read

Boundary replica count 2 is a real recall-recovery lever at 50k f8. The b2
cells hit recall 1.0000 at nprobe 96, and b2/tr50 improves low-nprobe recall
over b2/tr10. The tradeoff is latency: b2/tr50 is slower at most fixed nprobe
points, with p50 1896.008 ms at nprobe 96 versus 1743.187 ms for b2/tr10.

B4/tr10 improves recall over b2/tr10 at low and mid nprobe, reaching recall
1.0000 by nprobe 64 instead of nprobe 96. The tradeoff is larger storage and
higher fixed-nprobe latency.

B4/tr50 is the strongest 50k recall cell in this packet. It improves low/mid
nprobe recall over b4/tr10 and reaches recall 1.0000 by nprobe 48. The tradeoff
is b4-sized storage and higher fixed-nprobe latency than the b2 cells; b4/tr50
is also slower than b4/tr10 at most fixed nprobe points.

This packet does not close Task 121; the full 100k matrix, credible clean
latency, Phase 3 scan-efficiency A/B, and Phase 4 verdict are still owed.
