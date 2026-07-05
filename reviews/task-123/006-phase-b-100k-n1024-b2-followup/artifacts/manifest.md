# Task 123 Phase B 100k nlists=1024 Boundary=2 Follow-up Manifest

- Head SHA: `7b2234426af01b6126ec6cc2869654813f25bdff`
- Task bucket: `reviews/task-123/006-phase-b-100k-n1024-b2-followup`
- Timestamp: `2026-06-27T17:01:55Z`
- Lane: local PG18, database `tqvector_bench_task121`, socket `/tmp`, port `28818`
- Fixture: staged real corpus 100k (`/home/peter/dev/ecaz/data/staged-current/ec_real_100k_*`)
- Surface isolation: one table and one `ec_spire` index for the tested cell:
  - `t123_p6_100k_n1024_b2_tr50_f8`
- Storage format / rerank mode: `storage_format=rabitq`, SPIRE default rerank (`rerank_width=0` in cost snapshot)
- Variant axis: `nlists=1024`, `boundary_replica_count=2`, `training_sample_rows=50000`, `recursive_fanout=8`
- Suite config: `task123-phase-b-100k-n1024-b2-followup-suite.json`
- Suite manifest: `suite-manifest.json`
- Normalized results: `suite-results.jsonl`

## Commands

Audit:

```sh
/home/peter/dev/ecaz/target/debug/ecaz bench suite audit \
  --config reviews/task-123/006-phase-b-100k-n1024-b2-followup/artifacts/task123-phase-b-100k-n1024-b2-followup-suite.json
```

Run:

```sh
/home/peter/dev/ecaz/target/debug/ecaz bench suite run \
  --config reviews/task-123/006-phase-b-100k-n1024-b2-followup/artifacts/task123-phase-b-100k-n1024-b2-followup-suite.json \
  --database tqvector_bench_task121 \
  --host /tmp \
  --port 28818 \
  --manifest-output reviews/task-123/006-phase-b-100k-n1024-b2-followup/artifacts/suite-manifest.json \
  --results-output reviews/task-123/006-phase-b-100k-n1024-b2-followup/artifacts/suite-results.jsonl \
  --allow-debug-backend
```

## Artifacts

- `task123-phase-b-100k-n1024-b2-followup-suite.json`: `ecaz bench suite` config.
- `suite-manifest.json`: structured run manifest emitted by the suite runner.
- `suite-results.jsonl`: normalized result rows emitted by the suite runner.
- `load-100k-n1024-b2-tr50-f8.log`: corpus load, encoding, and index build log.
- `storage-100k-n1024-b2-tr50-f8.log`: storage size output.
- `latency-flat-floor-100k-repeat.log`: repeated 100k flat exact floor with index scans disabled.
- `latency-spire-100k-n1024-b2-nprobe-8-16-32-64.log`: clean cache-warm SPIRE latency.
- `spire-pipeline-100k-n1024-b2-nprobe-8-16-32-64.log`: query metrics, recall, route/candidate counters, and local-store overlap.
- `funnel-100k-n1024-b2-nprobe-8-16-32-64.jsonl`: per-query funnel counters and timing.
- `stage-containment-100k-n1024-b2-nprobe-8-16-32-64.jsonl`: per-query route/stage truth-containment records.

## Key Results

The repeated 100k flat exact floor measured p50/p95 `161.1 / 237.7 ms`.

Clean latency from `suite-results.jsonl`:

| Config | nprobe | p50 | p95 | Ratio vs repeated flat p50 |
| --- | ---: | ---: | ---: | ---: |
| n1024 b2 | 8 | 120.1 ms | 153.0 ms | 0.75x |
| n1024 b2 | 16 | 179.6 ms | 220.9 ms | 1.12x |
| n1024 b2 | 32 | 312.3 ms | 449.2 ms | 1.94x |
| n1024 b2 | 64 | 526.0 ms | 644.8 ms | 3.26x |

Coordinator query metrics from `spire-pipeline-100k-n1024-b2-nprobe-8-16-32-64.log`:

| Config | nprobe | Pipeline p50 | Pipeline p95 | Recall@10 |
| --- | ---: | ---: | ---: | ---: |
| n1024 b2 | 8 | 114.018 ms | 142.807 ms | 0.8375 |
| n1024 b2 | 16 | 171.233 ms | 222.823 ms | 0.9125 |
| n1024 b2 | 32 | 297.638 ms | 368.303 ms | 0.9437 |
| n1024 b2 | 64 | 576.094 ms | 656.251 ms | 0.9656 |

Route containment and final recall, aggregated from
`stage-containment-100k-n1024-b2-nprobe-8-16-32-64.jsonl`:

| Config | nprobe | Route containment | Final recall basis | Recall@10 |
| --- | ---: | ---: | ---: | ---: |
| n1024 b2 | 8 | 268 / 320 | 268 / 320 | 0.8375 |
| n1024 b2 | 16 | 292 / 320 | 292 / 320 | 0.9125 |
| n1024 b2 | 32 | 302 / 320 | 302 / 320 | 0.9438 |
| n1024 b2 | 64 | 309 / 320 | 309 / 320 | 0.9656 |

Candidate and local-store volume from `suite-results.jsonl`:

| Config | nprobe | Candidates/query | Object bytes/query |
| --- | ---: | ---: | ---: |
| n1024 b2 | 8 | 2,528 | 2.0 MiB |
| n1024 b2 | 16 | 4,779 | 3.7 MiB |
| n1024 b2 | 32 | 9,194 | 7.2 MiB |
| n1024 b2 | 64 | 18,416 | 14.3 MiB |

Storage from `storage-100k-n1024-b2-tr50-f8.log` and `suite-results.jsonl`:

| Config | SPIRE index size | All indexes | Total table |
| --- | ---: | ---: | ---: |
| n1024 b2 | 246.0 MiB | 248.3 MiB | 1.8 GiB |

## Interpretation

`boundary_replica_count=2` improves route containment over packet 004's b0/b1
spot-check, but the added recall is not enough to reach the requested
high-recall operating region. Even at `nprobe=64`, b2 reaches only
`309/320 = 0.9656` recall while clean p50 is `526.0 ms`, or `3.26x` the
same-run repeated flat p50.

The cost is also material: the b2 SPIRE index is `246.0 MiB`, compared with
packet 004's `167.9 MiB` for b1 and `89.8 MiB` for b0. At nprobe 32, b2 only
adds four recalled truths over b1 (`302/320` vs `298/320`) while clean p50 rises
from `236.1 ms` to `312.3 ms` and the index grows by about `78 MiB`.

Route containment still equals final recall for every b2 row, so the remaining
loss is still route selection rather than candidate scoring or rerank loss.
This follow-up therefore reinforces the Task 123 no-go / re-scope conclusion:
finer `nlists=1024` leaves plus boundary replication reduce scan cost versus
the original `nlists=128` high-recall path, but do not recover route containment
near 0.99 before latency and storage costs become unattractive.
