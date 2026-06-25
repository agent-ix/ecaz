# Task 121 Packet 022 50k Retuned Pruning Latency/Pipeline Summary

## Scope

This packet measures the packet 021 recall-neutral sampled global pruning
retune on the packet 020 50k b4/tr50/f8 RaBitQ block-summary index:

```text
max_global_blocks=2048
global_probe_blocks=4096
sample_rows_per_block=4
sample_summary_prior_weight=0.8
summary_radius_weight=0.25
route_prior_weight=0.0
```

The suite ran storage, a packet-local q200/k10 truth-cache seed, standalone
latency, and SPIRE pipeline A/B for pruning off versus the retuned policy at
nprobe 48, 64, and 96.

## Storage

The reused 50k index shape remains:

```text
index: 203.4 MiB, 4265.2 B/row
total: 998.4 MiB, 20936.9 B/row
```

## Recall

The packet-local truth-cache seed at nprobe 96 saturated recall:

```text
nprobe=96 recall@10=1.0000 mean_q_time=2029.71 ms
```

Both pipeline A/B policies preserved recall at all measured nprobes:

| nprobe | off pipeline recall@10 | retuned sampled pipeline recall@10 |
|---:|---:|---:|
| 48 | 1.0000 | 1.0000 |
| 64 | 1.0000 | 1.0000 |
| 96 | 1.0000 | 1.0000 |

## Standalone Latency

| nprobe | off p50 | retuned sampled p50 | p50 delta | off p95 | retuned sampled p95 | p95 delta |
|---:|---:|---:|---:|---:|---:|---:|
| 48 | 1406.6 ms | 1411.5 ms | +4.9 ms | 1691.4 ms | 1643.9 ms | -47.5 ms |
| 64 | 1581.1 ms | 1611.8 ms | +30.7 ms | 1925.1 ms | 1908.2 ms | -16.9 ms |
| 96 | 1988.0 ms | 1807.6 ms | -180.4 ms | 2340.2 ms | 2010.6 ms | -329.6 ms |

## Pipeline

| nprobe | off p50 | retuned sampled p50 | p50 delta | off candidates | retuned sampled candidates | candidate delta |
|---:|---:|---:|---:|---:|---:|---:|
| 48 | 1409.952 ms | 1384.971 ms | -24.981 ms | 19,807,598 | 19,807,598 | 0 |
| 64 | 1666.490 ms | 1548.382 ms | -118.108 ms | 26,111,939 | 26,521,536 | +409,597 |
| 96 | 2073.081 ms | 1737.456 ms | -335.625 ms | 37,774,415 | 28,367,005 | -9,407,410 |

Object-byte counters were unchanged in this local pipeline surface:

| nprobe | off object bytes | retuned sampled object bytes |
|---:|---:|---:|
| 48 | 16,649,378,828 | 16,649,378,828 |
| 64 | 21,948,603,032 | 21,948,603,032 |
| 96 | 31,752,679,906 | 31,752,679,906 |

## Interpretation

The retuned policy is recall-neutral at the measured 50k saturated checkpoints.
It is not a broad low/mid-nprobe win: standalone latency is effectively flat at
48 and slightly worse at 64. The useful signal is at nprobe 96, where standalone
p50 improves by 180.4 ms and pipeline p50 improves by 335.625 ms while reducing
candidate count by 9.4M over 200 queries.

The unchanged object-byte counters mean this local surface does not prove a
storage-read reduction for the retuned policy. It does prove a high-nprobe
candidate/latency reduction without the packet 020 recall loss.
