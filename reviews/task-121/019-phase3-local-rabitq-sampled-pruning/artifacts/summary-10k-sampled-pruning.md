# Task 121 Packet 019 10k Sampled Pruning Summary

## Scope

This packet is a Phase 3 pilot slice for the local RaBitQ block-summary pruning
surface. It tests the b4/tr50/f8 candidate at 10k with:

- `storage_format=rabitq`
- `leaf_block_rows=64`
- `leaf_block_summary_representatives=2`
- baseline pruning off
- sampled global pruning on with `max_global_blocks=384`,
  `global_probe_blocks=768`, `sample_rows_per_block=4`,
  `sample_summary_prior_weight=0.8`, `summary_radius_weight=0.25`, and
  `route_prior_weight=0.0`

The suite config also defines 50k and 100k cells, but the run was intentionally
interrupted after the completed 10k slice. The status log records 10 completed,
0 failed, and 18 pending/stale steps.

## Storage

The 10k summary-enabled RaBitQ index is storage-heavy:

```text
index=42.1 MiB
index_per_row=4415.5 B
table_heap_toast_fsm_vm=158.8 MiB
total=201.2 MiB
total_per_row=21094.4 B
```

## Recall

Sampled global pruning preserved recall at every tested nprobe.

| nprobe | off recall@10 | sampled recall@10 | off mean q-time | sampled mean q-time |
|---:|---:|---:|---:|---:|
| 8 | 0.9945 | 0.9945 | 70.23 ms | 71.25 ms |
| 16 | 0.9980 | 0.9980 | 109.06 ms | 108.25 ms |
| 32 | 0.9985 | 0.9985 | 149.66 ms | 151.37 ms |
| 48 | 0.9995 | 0.9995 | 205.82 ms | 210.88 ms |
| 64 | 1.0000 | 1.0000 | 262.34 ms | 250.64 ms |
| 96 | 1.0000 | 1.0000 | 342.01 ms | 288.73 ms |

## Latency

The p50 latency result is neutral to slightly worse at low nprobe, then
improves at the recall-saturated high nprobe points.

| nprobe | off p50 | sampled p50 | p50 delta |
|---:|---:|---:|---:|
| 8 | 68.4 ms | 72.8 ms | +4.4 ms |
| 16 | 102.7 ms | 102.5 ms | -0.2 ms |
| 32 | 144.0 ms | 146.6 ms | +2.6 ms |
| 48 | 202.9 ms | 210.7 ms | +7.8 ms |
| 64 | 254.0 ms | 248.6 ms | -5.4 ms |
| 96 | 340.5 ms | 285.6 ms | -54.9 ms |

The sampled p99 at nprobe 64 was noisy (`437.3 ms`), so the main signal here
is the high-nprobe p50/candidate-count reduction rather than a blanket latency
win.

## Pipeline Counters

At nprobe 96, sampled pruning cut local candidates and heap-rerank rows from
7,463,419 to 5,121,349 while preserving recall 1.0000. Object bytes did not
drop in this local scan path.

| nprobe | off candidates | sampled candidates | off p50 | sampled p50 | recall |
|---:|---:|---:|---:|---:|---:|
| 32 | 2,640,327 | 2,640,327 | 146.245 ms | 145.042 ms | 0.9985 |
| 48 | 3,832,280 | 3,832,280 | 203.235 ms | 207.622 ms | 0.9995 |
| 64 | 5,024,981 | 4,923,248 | 257.308 ms | 245.439 ms | 1.0000 |
| 96 | 7,463,419 | 5,121,349 | 342.133 ms | 283.740 ms | 1.0000 |

Object bytes were unchanged at nprobe 32/48/64/96:

```text
32: 2224632834
48: 3229381306
64: 4234411412
96: 6288592240
```

## Interpretation

This pilot gives a usable first Phase 3 signal: sampled global block pruning can
recover high-nprobe scan efficiency without losing recall on the 10k local
RaBitQ slice. It is not enough to close Task 121 because 50k/100k A/B evidence
is still required, and the current implementation still limits this pruning
surface to RaBitQ summary payloads rather than the default/TurboQuant storage
surface.
