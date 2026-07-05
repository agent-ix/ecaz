# Task 111h Packet 040: 50k Cold-Start Candidate Summary

Head SHA: `9b6f91a8dedc124c0f27fe062285ed3c77c0b4a7`

Suite: `task111h-50k-cold-start-candidates`

Fixture: `data/staged-current/ec_real_50k_*`, `dim=1536`, `k=10`, `queries=200`, `nlists=256`, `nprobe=32/128/200`, PG18 socket `/home/peter/.pgrx`, database `task111h_cold_50k`.

This is a single-query cold-start probe, not a latency distribution. Each latency step used `iterations=1`, `concurrency=1`, and `cache_state=relation_files_evicted_before_step`; therefore p50/p95/p99 equal the one observed query time. The suite inserted an `ecaz dev evict-relation-cache --prefix ...` step immediately before each latency step. The per-step eviction log files are 0-byte in this run; the suite manifest/status prove those commands completed, and post-run dry-run probes prove the relation/file coverage that the helper resolves.

| Variant | Rerank placement/format | Width/knobs | Recall@10 np32 / np128 / np200 | NDCG@10 np32 / np128 / np200 | Single-query cold latency np32 / np128 / np200 | IVF index | Total relation footprint | Dry-run resolved eviction footprint |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `source-f32-w32` | `source/f32` | width 32 | 0.9520 / 0.9875 / 0.9895 | 0.9973 / 0.9998 / 0.9999 | 5.91 ms / 9.99 ms / 12.1 ms | 13.8 MiB | 808.7 MiB | 5 relations, 9 files, 848027648 bytes |
| `index-f16-w32` | `index/f16` | width 32 | 0.9520 / 0.9875 / 0.9895 | 0.9973 / 0.9998 / 0.9999 | 21.7 ms / 10.1 ms / 12.5 ms | 172.5 MiB | 967.4 MiB | 5 relations, 9 files, 1014423552 bytes |
| `index-rabitq4-w128` | `index/rabitq4` | width 128 | 0.9200 / 0.9450 / 0.9460 | 0.9970 / 0.9994 / 0.9995 | 17.5 ms / 12.8 ms / 13.7 ms | 54.0 MiB | 848.9 MiB | 5 relations, 9 files, 890134528 bytes |
| `index-rabitq8-c4-w64` | `index/rabitq8` | width 64, `rabitq_rerank_least_squares=0`, `rabitq_rerank_clip=4` | 0.9550 / 0.9915 / 0.9930 | 0.9974 / 0.9999 / 1.0000 | 7.84 ms / 14.1 ms / 13.3 ms | 93.4 MiB | 888.3 MiB | 5 relations, 9 files, 931487744 bytes |
| `index-turboquant-w32` | `index/turboquant` | width 32 | 0.9300 / 0.9550 / 0.9565 | 0.9971 / 0.9996 / 0.9997 | 7.74 ms / 10.9 ms / 12.7 ms | 62.3 MiB | 857.1 MiB | 5 relations, 9 files, 898826240 bytes |

Interpretation:

- `source/f32` remains the strongest default candidate in this run: smallest IVF index, high recall, and the lowest or tied cold latency samples.
- Current `index/f16` is recall-neutral versus `source/f32` but much larger on disk and has no observed cold-start advantage in this probe.
- `index/rabitq8` with clip 4 materially improves recall over earlier default RaBitQ8 evidence, reaching 0.9915 at nprobe 128 and 0.9930 at nprobe 200 on 50k. It is still larger and not faster than `source/f32` in these single-query cold samples.
- `index/rabitq4` remains below 0.95 recall at nprobe 200 on this fixture.
- `index/turboquant` is smaller than clipped RaBitQ8 but does not recover source-quality recall here.

Evidence caveats:

- The suite evicts local PostgreSQL relation files via `posix_fadvise(POSIX_FADV_DONTNEED)`. It is a local OS page-cache hint, not a remote-storage simulation.
- The query table/truth cache path is not part of the per-prefix relation eviction.
- The cold latency results are one sample per nprobe and are useful for detecting order-of-magnitude failures, not for stable percentile comparisons.
- `reviews/task-111h/040-cold-cache-50k-candidates/artifacts/suite/truth-50k-k10.json` is a generated ground-truth cache and is intentionally not committed.
