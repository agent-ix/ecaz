# Task 121 Packet 018 Summary

## Scope

Local PG18 follow-up for the 100k Phase 2 boundary knee after packet 017.
This packet adds b8/tr50 saturation evidence for fanout 8 and 16, and clean
cache-warm latency remeasurement for the b4 and b8 finalist cells.

Fixed settings:

- `nlists=128`
- `storage_format=rabitq`
- top graph enabled, degree 32, build list size 100, search list size 96
- training sample rows 50000
- boundary replica count 8 for new pipeline/storage rows
- one table/index per cell; no shared-table surface

## Pipeline Recall

| cell | r@4 | r@8 | r@12 | r@16 | r@24 | r@32 | r@48 | r@64 | r@96 | p50@32 | p50@96 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| b8_tr50_f8 | 0.8980 | 0.9630 | 0.9785 | 0.9830 | 0.9945 | 0.9970 | 0.9985 | 1.0000 | 1.0000 | 3477.841 ms | 5032.966 ms |
| b8_tr50_f16 | 0.8995 | 0.9680 | 0.9815 | 0.9900 | 0.9955 | 0.9980 | 0.9990 | 1.0000 | 1.0000 | 3528.944 ms | 5058.282 ms |

Interpretation: b8 reaches recall 1.0000 by nprobe 64 for both fanouts, but
the improvement over packet 017 b4/tr50 is small relative to storage and
latency cost. b8/f16 is slightly better than b8/f8 at most recall checkpoints,
but the gain is not large enough to change the boundary-cost conclusion.

## Clean Cache-Warm Latency

| cell | p50@8 | p50@16 | p50@32 | p50@48 | p50@96 |
|---|---:|---:|---:|---:|---:|
| b4_tr50_f8 | 955.0 ms | 1629.8 ms | 2699.4 ms | 3451.9 ms | 4730.6 ms |
| b4_tr50_f16 | 984.3 ms | 1660.4 ms | 2732.8 ms | 3497.3 ms | 4708.2 ms |
| b8_tr50_f8 | 1681.7 ms | 2661.6 ms | 3595.2 ms | 4250.1 ms | 5215.1 ms |
| b8_tr50_f16 | 1673.1 ms | 2565.5 ms | 3624.8 ms | 4294.1 ms | 5283.8 ms |

Interpretation: b8 roughly doubles low-nprobe p50 latency versus b4 and stays
slower at high nprobe. The b4/f8 and b4/f16 clean latency rows are close to
each other; b8/f16 is not a clean latency win over b8/f8.

## Storage

| cell | index size | index bytes/row | total table |
|---|---:|---:|---:|
| b8_tr50_f8 | 704.7 MiB | 7389.3 B | 2.2 GiB |
| b8_tr50_f16 | 704.8 MiB | 7390.0 B | 2.2 GiB |

Interpretation: b8 is about 1.8x the b4 index footprint from packet 017
(`~392.2-392.3 MiB`) for only modest recall improvement at the 100k knee.

## Diagnostic Shape

Both b8 pipeline steps wrote the expected diagnostic shape:

```text
funnel_rows=1800
stage_containment_rows=10800
```

The streamed `pipeline-100k_b8_tr50_f*-funnel.jsonl`,
`pipeline-100k_b8_tr50_f*-stage-containment.jsonl`, and
`truth-cache-100k-q200-k10.json` files are local-only diagnostics and are not
commit artifacts.
