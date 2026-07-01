# Extracted Results

- Head SHA: `fe57bb57d291123873819330d206acea3d2b8a14`
- Task bucket: `reviews/task-123/009-multi-instance-phase-a-baseline/`
- Lane: contained local multi-instance, one coordinator plus three worker PG18 instances on one host
- Fixture: staged representative 100k corpus, `ec_real_100k`, corpus SHA `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95`, queries SHA `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Query sample: 32 queries, `top_k=10`
- Storage format: `rabitq`
- Transport status: `pg_binary_attr_v1` ready

## Coordinator Query Metrics

| Config | nprobe | Recall@10 | Latency p50 | Latency p95 | Artifact |
| --- | ---: | ---: | ---: | ---: | --- |
| n128 b4/tr50/f8 | 8 | 0.9781 | 69.620 ms | 78.007 ms | `n128-b4-r2/bench-suite/production-read-k10-default.log` |
| n128 b4/tr50/f8 | 96 | 1.0000 | 337.096 ms | 479.785 ms | `n128-b4-r2/bench-suite/production-read-k10-default.log` |
| n1024 b2/tr50/f8 | 8 | 0.9406 | 75.196 ms | 85.457 ms | `n1024-b2-r3/bench-suite/production-read-k10-default.log` |
| n1024 b2/tr50/f8 | 64 | 1.0000 | 87.323 ms | 90.365 ms | `n1024-b2-r3/bench-suite/production-read-k10-default.log` |

The rowcap25k variant was effectively neutral in this production-read lane:

| Config | nprobe | Recall@10 | Rowcap p50 | Rowcap p95 |
| --- | ---: | ---: | ---: | ---: |
| n128 b4/tr50/f8 | 8 | 0.9781 | 69.991 ms | 85.777 ms |
| n128 b4/tr50/f8 | 96 | 1.0000 | 340.505 ms | 374.994 ms |
| n1024 b2/tr50/f8 | 8 | 0.9406 | 75.161 ms | 83.129 ms |
| n1024 b2/tr50/f8 | 64 | 1.0000 | 85.816 ms | 91.714 ms |

## Production-Read Profile

| Config | nprobe | Result source | Selected pids | Remote pids | Dispatches | Remote heap candidates | Candidate p50/p95 | Heap p50/p95 | Total p50/p95 | Payload rows | Payload bytes |
| --- | ---: | --- | ---: | ---: | ---: | ---: | --- | --- | --- | ---: | ---: |
| n128 b4/tr50/f8 | 8 | remote_heap_candidates | 256 | 256 | 95 | 950 | 34/41 ms | 34/39 ms | 62/74 ms | 950 | 0 |
| n128 b4/tr50/f8 | 96 | remote_heap_candidates | 3072 | 3072 | 96 | 960 | 386/545 ms | 400/539 ms | 339/437 ms | 960 | 0 |
| n1024 b2/tr50/f8 | 8 | remote_heap_candidates | 256 | 256 | 93 | 930 | 6/7 ms | 6/7 ms | 52/54 ms | 930 | 0 |
| n1024 b2/tr50/f8 | 64 | remote_heap_candidates | 2048 | 2048 | 96 | 960 | 20/24 ms | 20/24 ms | 63/68 ms | 960 | 0 |

The profile timing buckets are aggregate profile percentiles, not additive
sub-stages; they should be read as attribution signals, not summed into the
coordinator query p50.

`payload_bytes_sum=0` is not an object-byte measurement. The nested local-multinode suite sets `query_metric_projection_columns=["id"]`, so the current production-read profile reports projected payload bytes, not object bytes shipped per worker. This packet therefore records `payload_rows_sum`, remote dispatch/candidate counts, and candidate/heap/total timings, and leaves per-worker object-byte attribution as an instrumentation gap.

## Coordinator Storage

| Config | Coordinator index | Index size | Per row | Artifact |
| --- | --- | ---: | ---: | --- |
| n128 b4/tr50/f8 | `t123_p9_mi_100k_n128_b4_coord_idx` | 392.2 MiB | 4112.6 B | `n128-b4-r2/bench-suite/storage.log` |
| n1024 b2/tr50/f8 | `t123_p9_mi_100k_n1024_b2_coord_idx` | 246.1 MiB | 2580.9 B | `n1024-b2-r3/bench-suite/storage.log` |

## Fixture Load Notes

| Config | Coordinator load | Remote node 2 | Remote node 3 | Remote node 4 |
| --- | ---: | ---: | ---: | ---: |
| n128 b4/tr50/f8 | 399.92 s | 89,420 rows / 324.85 s | 89,083 rows / 310.46 s | 84,793 rows / 299.10 s |
| n1024 b2/tr50/f8 | 703.56 s | 69,835 rows / 478.26 s | 71,040 rows / 476.74 s | 70,575 rows / 466.22 s |

The generated shard TSVs and local PG runtime directories were removed after the run. They are regenerable corpus/runtime data and are banned from commits by repository policy.
