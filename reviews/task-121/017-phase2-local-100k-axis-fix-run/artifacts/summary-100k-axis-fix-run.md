# Task 121 Phase 2 Local 100k Axis-Fix Summary

## Matrix

- Scale: 100k real corpus
- Queries: 200 for each pipeline sweep
- Sweep: nprobe `4,8,12,16,24,32,48,64,96`
- Cells: boundary replica count `0,1,2,4` x training sample rows
  `10000,50000` x recursive fanout `8,16`
- Fixed: `nlists=128`, top graph enabled, top graph degree 32, top graph
  search list size 96, `storage_format=rabitq`

## Storage

| cell | index | index bytes/row | total table |
|---|---:|---:|---:|
| b0_tr10_f8 | 79.7 MiB | 835.8 B | 1.6 GiB |
| b0_tr10_f16 | 79.8 MiB | 836.6 B | 1.6 GiB |
| b0_tr50_f8 | 79.6 MiB | 835.2 B | 1.6 GiB |
| b0_tr50_f16 | 79.7 MiB | 835.8 B | 1.6 GiB |
| b1_tr10_f8 | 157.9 MiB | 1655.2 B | 1.7 GiB |
| b1_tr10_f16 | 157.9 MiB | 1656.0 B | 1.7 GiB |
| b1_tr50_f8 | 157.8 MiB | 1654.5 B | 1.7 GiB |
| b1_tr50_f16 | 157.8 MiB | 1655.1 B | 1.7 GiB |
| b2_tr10_f8 | 235.9 MiB | 2473.8 B | 1.8 GiB |
| b2_tr10_f16 | 236.0 MiB | 2474.6 B | 1.8 GiB |
| b2_tr50_f8 | 235.9 MiB | 2474.1 B | 1.8 GiB |
| b2_tr50_f16 | 236.0 MiB | 2474.7 B | 1.8 GiB |
| b4_tr10_f8 | 392.2 MiB | 4112.4 B | 1.9 GiB |
| b4_tr10_f16 | 392.3 MiB | 4113.2 B | 1.9 GiB |
| b4_tr50_f8 | 392.2 MiB | 4112.5 B | 1.9 GiB |
| b4_tr50_f16 | 392.3 MiB | 4113.1 B | 1.9 GiB |

## Selected Recall And Latency

Full per-nprobe rows are in `suite-phase2-local-100k-axis-fix-run-results.jsonl`
and `pipeline-100k_*.log`. This table keeps the review surface compact.

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

## Interpretation

- Boundary replication is the decisive 100k route-recall lever in this grid.
  Moving from b0 to b4 adds roughly 0.16-0.21 recall at nprobe 8, depending on
  training and fanout.
- Training sample rows 50000 improve b0/b1/b2 low- and mid-nprobe recall
  relative to 10000. The b4/tr50 cells are the best low-nprobe cells, but the
  incremental gain over b4/tr10 is smaller than the b0-to-b4 boundary effect.
- Recursive fanout 16 is not a decisive win. It improves selected recall points
  for some cells, but fanout 8 is often similar and sometimes slightly better.
- The b4 cells recover recall, but the cost is visible: b4 storage is about
  4.9x b0 index size, and p50 latency at nprobe 32 is about 2.4x the b0
  baseline.
- This points Phase 3 at scan-efficiency work for b4/tr50-style candidates,
  not more route-recall tuning in this specific grid.
